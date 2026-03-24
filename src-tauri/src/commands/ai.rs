use tauri::State;

use cheshire_ai::{GlossaryContext, Suggestion, SuggestionRequest, TmContext};

use crate::state::AppState;

// ── Request / response types ──────────────────────────────────────────────────

/// Serialisable mirror of `TmContext` so the frontend can pass TM hits in.
#[derive(serde::Deserialize)]
pub struct TmHit {
    pub source: String,
    pub target: String,
    pub score: u8,
}

/// Serialisable mirror of `GlossaryContext`.
#[derive(serde::Deserialize)]
pub struct GlossaryHit {
    pub source_term: String,
    pub target_term: String,
}

// ── Commands ──────────────────────────────────────────────────────────────────

/// Ask the AI orchestrator for a translation suggestion.
///
/// Returns `null` when the AI is disabled, the best TM score is below the
/// configured `min_tm_score` threshold, or the network call fails gracefully.
#[tauri::command]
pub async fn ai_suggest(
    state: State<'_, AppState>,
    source: String,
    source_lang: String,
    target_lang: String,
    tm_matches: Vec<TmHit>,
    glossary_hits: Vec<GlossaryHit>,
    domain: Option<String>,
    prev_target: Option<String>,
) -> Result<Option<Suggestion>, String> {
    let req = SuggestionRequest {
        source,
        source_lang,
        target_lang,
        tm_matches: tm_matches
            .into_iter()
            .map(|h| TmContext {
                source: h.source,
                target: h.target,
                score: h.score,
            })
            .collect(),
        glossary_hits: glossary_hits
            .into_iter()
            .map(|h| GlossaryContext {
                source_term: h.source_term,
                target_term: h.target_term,
            })
            .collect(),
        domain,
        prev_target,
    };

    state
        .orchestrator
        .get_draft(&req)
        .await
        .map_err(|e| e.to_string())
}
