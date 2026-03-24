use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use dashmap::DashMap;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::prompt::build_prompt;
use crate::{AiError, Result};

// ── Public types ──────────────────────────────────────────────────────────────

/// Which AI backend to use for translation suggestions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AiMode {
    /// Native Ollama API (`/api/generate`). Local GPU/CPU inference.
    #[default]
    Ollama,
    /// Any OpenAI-compatible API (`/v1/chat/completions`).
    ///
    /// Works with Ollama in OpenAI mode, LM Studio, Azure OpenAI,
    /// and hosted services that mirror the OpenAI chat completions format.
    OpenAiCompatible,
    /// vLLM inference server (`/v1/chat/completions`).
    ///
    /// Identical wire format to `OpenAiCompatible`. Exposed as a separate
    /// variant so the UI can pre-fill `http://localhost:8000` and surface
    /// vLLM-specific guidance.
    Vllm,
    /// DeepL machine-translation API (`/v2/translate`).
    ///
    /// Bypasses the `min_tm_score` gate — DeepL is always called when in
    /// this mode. Set `base_url` to `https://api-free.deepl.com` (free tier)
    /// or `https://api.deepl.com` (paid). Authenticate via `api_key`.
    DeepL,
    /// AI suggestions disabled. `get_draft` returns `Ok(None)` immediately.
    Disabled,
}

/// Configuration for the AI orchestrator.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub mode: AiMode,
    /// Base URL of the AI backend (e.g. `"http://localhost:11434"`).
    pub base_url: String,
    /// Model identifier (e.g. `"llama3.2:3b"`, `"gpt-4o-mini"`).
    pub model: String,
    /// Bearer token for cloud APIs. Not required for local Ollama.
    pub api_key: Option<String>,
    /// Cache responses so identical prompts skip the network round-trip.
    pub cache_responses: bool,
    /// Only invoke AI when the best TM match scores at least this high (0–100).
    /// Set to 0 to call AI even when there is no TM context.
    pub min_tm_score: u8,
    /// HTTP request timeout in seconds.
    pub timeout_secs: u64,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            mode: AiMode::Ollama,
            base_url: "http://localhost:11434".into(),
            model: "llama3.2:3b".into(),
            api_key: None,
            cache_responses: true,
            min_tm_score: 0,
            timeout_secs: 60,
        }
    }
}

/// A TM match summary passed to the AI as translation evidence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TmContext {
    pub source: String,
    pub target: String,
    /// Match score 0–100.
    pub score: u8,
}

/// A glossary constraint the AI must honour.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlossaryContext {
    pub source_term: String,
    pub target_term: String,
}

/// Everything the AI needs to produce a translation suggestion.
///
/// The AI is only invoked when there is at least one TM match above
/// `AiConfig::min_tm_score` OR at least one glossary hit — never as a blind
/// machine-translation replacement unless `min_tm_score` is set to 0.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SuggestionRequest {
    pub source: String,
    pub source_lang: String,
    pub target_lang: String,
    /// Up to three best TM matches (score ≥60%), descending.
    pub tm_matches: Vec<TmContext>,
    /// All glossary hits found in the source segment.
    pub glossary_hits: Vec<GlossaryContext>,
    /// Domain hint for specialised terminology (e.g. `"legal"`, `"medical"`).
    /// Falls back to `"general"` when `None`.
    pub domain: Option<String>,
    /// Target text of the immediately preceding confirmed segment, for
    /// discourse context so the model maintains register and style.
    pub prev_target: Option<String>,
}

/// A translation suggestion returned by the AI.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    pub text: String,
    /// True when the model returned an empty or unparseable response.
    pub is_fallback: bool,
}

// ── DeepL wire types ──────────────────────────────────────────────────────────

#[derive(Serialize)]
struct DeepLRequest<'a> {
    text: Vec<&'a str>,
    target_lang: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    source_lang: Option<String>,
}

#[derive(Deserialize)]
struct DeepLResponse {
    translations: Vec<DeepLTranslation>,
}

#[derive(Deserialize)]
struct DeepLTranslation {
    text: String,
}

// ── Ollama wire types ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: String,
    stream: bool,
    options: OllamaOptions,
}

#[derive(Debug, Serialize)]
struct OllamaOptions {
    num_predict: u32,
    temperature: f32,
    top_p: f32,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
}

// ── Orchestrator ──────────────────────────────────────────────────────────────

/// Sends suggestion requests to a configured AI backend with in-process caching.
///
/// The orchestrator degrades gracefully when the AI is unreachable:
/// `get_draft` returns `Ok(None)` on connectivity errors so the UI can hide
/// the AI panel rather than show an error to the translator.
pub struct Orchestrator {
    client: Client,
    config: AiConfig,
    /// Prompt hash → model response. Only populated when `config.cache_responses`.
    cache: Arc<DashMap<u64, String>>,
}

impl Orchestrator {
    pub fn new(config: AiConfig) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(config.timeout_secs))
            .build()
            .expect("failed to build HTTP client");
        Self {
            client,
            config,
            cache: Arc::new(DashMap::new()),
        }
    }

    /// Hash the parts of a request that affect the model's output.
    fn cache_key(req: &SuggestionRequest) -> u64 {
        let mut h = DefaultHasher::new();
        req.source.hash(&mut h);
        req.source_lang.hash(&mut h);
        req.target_lang.hash(&mut h);
        req.domain.hash(&mut h);
        req.prev_target.hash(&mut h);
        for m in &req.tm_matches {
            m.source.hash(&mut h);
            m.score.hash(&mut h);
        }
        for g in &req.glossary_hits {
            g.source_term.hash(&mut h);
        }
        h.finish()
    }

    /// Request a translation draft.
    ///
    /// Returns `Ok(None)` when:
    /// - mode is `Disabled`
    /// - the best TM score is below `min_tm_score` and there are no glossary hits
    /// - the AI backend is unreachable or returns an error
    pub async fn get_draft(&self, req: &SuggestionRequest) -> Result<Option<Suggestion>> {
        if matches!(self.config.mode, AiMode::Disabled) {
            return Ok(None);
        }

        // DeepL bypasses the TM score gate — it is always invoked when selected
        if matches!(self.config.mode, AiMode::DeepL) {
            if self.config.cache_responses {
                let key = Self::cache_key(req);
                if let Some(cached) = self.cache.get(&key) {
                    return Ok(Some(Suggestion { text: cached.clone(), is_fallback: false }));
                }
            }
            return match self.call_deepl(req).await {
                Ok(text) => {
                    if self.config.cache_responses && !text.is_empty() {
                        self.cache.insert(Self::cache_key(req), text.clone());
                    }
                    Ok(Some(Suggestion { is_fallback: text.is_empty(), text }))
                }
                Err(_) => Ok(None),
            };
        }

        // Skip AI call when there is no useful context
        if self.config.min_tm_score > 0 {
            let best = req.tm_matches.iter().map(|m| m.score).max().unwrap_or(0);
            if best < self.config.min_tm_score && req.glossary_hits.is_empty() {
                return Ok(None);
            }
        }

        // Serve from cache when an identical prompt was already answered
        if self.config.cache_responses {
            let key = Self::cache_key(req);
            if let Some(cached) = self.cache.get(&key) {
                return Ok(Some(Suggestion {
                    text: cached.clone(),
                    is_fallback: false,
                }));
            }
        }

        let prompt = build_prompt(req);

        let result = match self.config.mode {
            AiMode::Ollama => self.call_ollama(&prompt).await,
            AiMode::OpenAiCompatible | AiMode::Vllm => self.call_openai_compatible(&prompt, req).await,
            AiMode::DeepL => self.call_deepl(req).await,
            AiMode::Disabled => return Ok(None),
        };

        match result {
            Ok(text) => {
                if self.config.cache_responses && !text.is_empty() {
                    self.cache.insert(Self::cache_key(req), text.clone());
                }
                Ok(Some(Suggestion {
                    is_fallback: text.is_empty(),
                    text,
                }))
            }
            // Connectivity or model error — degrade silently
            Err(_) => Ok(None),
        }
    }

    async fn call_ollama(&self, prompt: &str) -> Result<String> {
        let body = OllamaRequest {
            model: &self.config.model,
            prompt: prompt.to_string(),
            stream: false,
            options: OllamaOptions {
                num_predict: 256,
                temperature: 0.2,
                top_p: 0.9,
            },
        };

        let response = self
            .client
            .post(format!("{}/api/generate", self.config.base_url))
            .json(&body)
            .send()
            .await
            .map_err(|_| AiError::Request("Ollama unreachable".into()))?;

        if !response.status().is_success() {
            return Err(AiError::Request(format!(
                "Ollama HTTP {}",
                response.status()
            )));
        }

        let resp: OllamaResponse = response
            .json()
            .await
            .map_err(|e| AiError::Parse(e.to_string()))?;

        Ok(resp.response.trim().to_string())
    }

    async fn call_openai_compatible(&self, prompt: &str, req: &SuggestionRequest) -> Result<String> {
        let system = format!(
            "You are a professional translator from {src} to {tgt}. \
             Output only the translation, no commentary.",
            src = req.source_lang,
            tgt = req.target_lang,
        );

        let body = serde_json::json!({
            "model": self.config.model,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user",   "content": prompt },
            ],
            "max_tokens": 256,
            "temperature": 0.2,
        });

        let mut builder = self
            .client
            .post(format!("{}/v1/chat/completions", self.config.base_url))
            .json(&body);

        if let Some(ref key) = self.config.api_key {
            builder = builder.bearer_auth(key);
        }

        let response = builder
            .send()
            .await
            .map_err(|_| AiError::Request("AI backend unreachable".into()))?;

        if !response.status().is_success() {
            return Err(AiError::Request(format!(
                "AI backend HTTP {}",
                response.status()
            )));
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| AiError::Parse(e.to_string()))?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .trim()
            .to_string();

        Ok(content)
    }

    async fn call_deepl(&self, req: &SuggestionRequest) -> Result<String> {
        // DeepL language codes are uppercase (EN, JA, DE, etc.)
        let target_lang = req.target_lang.to_uppercase();
        // Only send source_lang when it's explicitly set (non-empty)
        let source_lang = if req.source_lang.is_empty() {
            None
        } else {
            Some(req.source_lang.to_uppercase())
        };

        let body = DeepLRequest {
            text: vec![&req.source],
            target_lang,
            source_lang,
        };

        // DeepL uses "DeepL-Auth-Key" header, not Bearer
        let mut builder = self
            .client
            .post(format!("{}/v2/translate", self.config.base_url))
            .json(&body);

        if let Some(ref key) = self.config.api_key {
            builder = builder.header("Authorization", format!("DeepL-Auth-Key {key}"));
        }

        let resp = builder
            .send()
            .await
            .map_err(|_| AiError::Request("DeepL API unreachable".into()))?;

        if !resp.status().is_success() {
            return Err(AiError::Request(format!("DeepL HTTP {}", resp.status())));
        }

        let parsed: DeepLResponse = resp
            .json()
            .await
            .map_err(|e| AiError::Parse(e.to_string()))?;

        Ok(parsed
            .translations
            .into_iter()
            .next()
            .map(|t| t.text)
            .unwrap_or_default())
    }

    /// Backward-compatible alias for `get_draft`.
    pub async fn suggest(&self, req: &SuggestionRequest) -> Result<Option<Suggestion>> {
        self.get_draft(req).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config_disabled() -> AiConfig {
        AiConfig {
            mode: AiMode::Disabled,
            ..Default::default()
        }
    }

    // ── AiMode ────────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn disabled_mode_returns_none() {
        let orc = Orchestrator::new(make_config_disabled());
        let req = SuggestionRequest {
            source: "Hello world.".into(),
            source_lang: "en".into(),
            target_lang: "ja".into(),
            ..Default::default()
        };
        let result = orc.get_draft(&req).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn min_tm_score_skips_ai_when_no_context() {
        let orc = Orchestrator::new(AiConfig {
            mode: AiMode::Ollama,
            min_tm_score: 70,
            ..Default::default()
        });
        let req = SuggestionRequest {
            source: "Hello world.".into(),
            source_lang: "en".into(),
            target_lang: "ja".into(),
            tm_matches: vec![TmContext {
                source: "Hi world.".into(),
                target: "こんにちは。".into(),
                score: 50, // below threshold
            }],
            glossary_hits: vec![], // no glossary hits either
            ..Default::default()
        };
        // Should return None without making any HTTP call
        let result = orc.get_draft(&req).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn min_tm_score_passes_when_glossary_hit_exists() {
        // Has glossary hit → should attempt AI call (which fails → Ok(None))
        let orc = Orchestrator::new(AiConfig {
            mode: AiMode::Ollama,
            min_tm_score: 70,
            base_url: "http://127.0.0.1:1".into(), // unreachable port
            ..Default::default()
        });
        let req = SuggestionRequest {
            source: "Sign the document.".into(),
            source_lang: "en".into(),
            target_lang: "ja".into(),
            tm_matches: vec![],
            glossary_hits: vec![GlossaryContext {
                source_term: "document".into(),
                target_term: "書類".into(),
            }],
            ..Default::default()
        };
        // Unreachable backend → graceful None, not an error
        let result = orc.get_draft(&req).await;
        assert!(result.is_ok());
        // None because backend is unreachable
        assert!(result.unwrap().is_none());
    }

    // ── Cache ─────────────────────────────────────────────────────────────────

    #[test]
    fn cache_key_same_request_produces_same_key() {
        let req = SuggestionRequest {
            source: "Sign here.".into(),
            source_lang: "en".into(),
            target_lang: "ja".into(),
            domain: Some("legal".into()),
            ..Default::default()
        };
        assert_eq!(
            Orchestrator::cache_key(&req),
            Orchestrator::cache_key(&req)
        );
    }

    #[test]
    fn cache_key_differs_on_source_change() {
        let base = SuggestionRequest {
            source: "Hello.".into(),
            source_lang: "en".into(),
            target_lang: "ja".into(),
            ..Default::default()
        };
        let changed = SuggestionRequest {
            source: "Goodbye.".into(),
            ..base.clone()
        };
        assert_ne!(Orchestrator::cache_key(&base), Orchestrator::cache_key(&changed));
    }

    #[test]
    fn cache_key_differs_on_domain_change() {
        let base = SuggestionRequest {
            source: "Sign here.".into(),
            source_lang: "en".into(),
            target_lang: "ja".into(),
            domain: Some("legal".into()),
            ..Default::default()
        };
        let no_domain = SuggestionRequest {
            domain: None,
            ..base.clone()
        };
        assert_ne!(Orchestrator::cache_key(&base), Orchestrator::cache_key(&no_domain));
    }

    #[test]
    fn cache_key_differs_on_prev_target_change() {
        let base = SuggestionRequest {
            source: "Second sentence.".into(),
            source_lang: "en".into(),
            target_lang: "ja".into(),
            prev_target: Some("First sentence translated.".into()),
            ..Default::default()
        };
        let no_prev = SuggestionRequest {
            prev_target: None,
            ..base.clone()
        };
        assert_ne!(Orchestrator::cache_key(&base), Orchestrator::cache_key(&no_prev));
    }

    // ── AiConfig defaults ─────────────────────────────────────────────────────

    #[test]
    fn default_config_is_ollama_mode() {
        let cfg = AiConfig::default();
        assert_eq!(cfg.mode, AiMode::Ollama);
    }

    #[test]
    fn default_config_has_cache_enabled() {
        assert!(AiConfig::default().cache_responses);
    }

    #[test]
    fn default_config_has_zero_min_score() {
        assert_eq!(AiConfig::default().min_tm_score, 0);
    }

    // ── Graceful degradation ──────────────────────────────────────────────────

    #[tokio::test]
    async fn unreachable_ollama_returns_ok_none() {
        let orc = Orchestrator::new(AiConfig {
            mode: AiMode::Ollama,
            base_url: "http://127.0.0.1:1".into(), // unreachable
            ..Default::default()
        });
        let req = SuggestionRequest {
            source: "Hello.".into(),
            source_lang: "en".into(),
            target_lang: "ja".into(),
            ..Default::default()
        };
        let result = orc.get_draft(&req).await;
        assert!(result.is_ok(), "should not return Err on connectivity failure");
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn unreachable_openai_returns_ok_none() {
        let orc = Orchestrator::new(AiConfig {
            mode: AiMode::OpenAiCompatible,
            base_url: "http://127.0.0.1:1".into(),
            ..Default::default()
        });
        let req = SuggestionRequest {
            source: "Hello.".into(),
            source_lang: "en".into(),
            target_lang: "ja".into(),
            ..Default::default()
        };
        let result = orc.get_draft(&req).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn deepl_mode_skips_min_score_gate() {
        // DeepL with unreachable backend → graceful Ok(None), score gate NOT applied
        let orc = Orchestrator::new(AiConfig {
            mode: AiMode::DeepL,
            base_url: "http://127.0.0.1:1".into(),
            min_tm_score: 90, // high score, no TM matches
            ..Default::default()
        });
        let req = SuggestionRequest {
            source: "Hello world.".into(),
            source_lang: "en".into(),
            target_lang: "ja".into(),
            tm_matches: vec![], // no matches
            glossary_hits: vec![],
            ..Default::default()
        };
        // Should attempt DeepL (not skip due to min_score gate) and degrade gracefully
        let result = orc.get_draft(&req).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // None because unreachable, not because gated
    }

    #[tokio::test]
    async fn deepl_mode_unreachable_returns_ok_none() {
        let orc = Orchestrator::new(AiConfig {
            mode: AiMode::DeepL,
            base_url: "http://127.0.0.1:1".into(),
            ..Default::default()
        });
        let req = SuggestionRequest {
            source: "Hello.".into(),
            source_lang: "en".into(),
            target_lang: "ja".into(),
            ..Default::default()
        };
        let result = orc.get_draft(&req).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }
}
