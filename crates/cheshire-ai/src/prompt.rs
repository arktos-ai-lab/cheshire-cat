use crate::orchestrator::SuggestionRequest;

/// Approximate character budget for the full prompt (≈512 tokens at 4 chars/token).
/// Kept well under 2048 to leave headroom for whitespace and formatting.
const CHAR_BUDGET: usize = 1800;

/// Build a compact prompt from the suggestion request.
///
/// The prompt always includes the preamble and source segment.  An optional
/// domain label and preceding-translation context are added when present.
/// TM matches and glossary terms are appended in descending priority until
/// the character budget is exhausted.
pub fn build_prompt(req: &SuggestionRequest) -> String {
    let mut buf = String::with_capacity(CHAR_BUDGET);
    let domain = req.domain.as_deref().unwrap_or("general");

    buf.push_str(&format!(
        "You are a professional translator from {} to {} ({} domain).\n\
         Produce ONE translation for the SOURCE SEGMENT.\n\
         Apply the TM MATCHES and GLOSSARY TERMS as hard constraints.\n\
         Output only the translated text — no explanations.\n\n",
        req.source_lang, req.target_lang, domain
    ));

    buf.push_str(&format!("SOURCE SEGMENT:\n{}\n\n", req.source));

    // Discourse context — the immediately preceding confirmed segment
    if let Some(ref prev) = req.prev_target {
        let header = "PRECEDING TRANSLATION:\n";
        let line = format!("{prev}\n\n");
        if buf.len() + header.len() + line.len() < CHAR_BUDGET {
            buf.push_str(header);
            buf.push_str(&line);
        }
    }

    if !req.tm_matches.is_empty() {
        let header = "TM MATCHES (source → target, score%):\n";
        if buf.len() + header.len() < CHAR_BUDGET {
            buf.push_str(header);
            for m in &req.tm_matches {
                let line = format!("[{}%] {} → {}\n", m.score, m.source, m.target);
                if buf.len() + line.len() >= CHAR_BUDGET {
                    break;
                }
                buf.push_str(&line);
            }
            buf.push('\n');
        }
    }

    if !req.glossary_hits.is_empty() {
        let header = "GLOSSARY TERMS (use exactly):\n";
        if buf.len() + header.len() < CHAR_BUDGET {
            buf.push_str(header);
            for h in &req.glossary_hits {
                let line = format!("{} → {}\n", h.source_term, h.target_term);
                if buf.len() + line.len() >= CHAR_BUDGET {
                    break;
                }
                buf.push_str(&line);
            }
            buf.push('\n');
        }
    }

    buf.push_str("TRANSLATION:\n");
    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::orchestrator::{GlossaryContext, TmContext};

    fn make_req() -> SuggestionRequest {
        SuggestionRequest {
            source: "Please sign the document.".into(),
            source_lang: "en".into(),
            target_lang: "ja".into(),
            tm_matches: vec![TmContext {
                source: "Sign the document".into(),
                target: "書類に署名する".into(),
                score: 82,
            }],
            glossary_hits: vec![GlossaryContext {
                source_term: "document".into(),
                target_term: "書類".into(),
            }],
            ..Default::default()
        }
    }

    fn empty_req() -> SuggestionRequest {
        SuggestionRequest {
            source: "Hello world.".into(),
            source_lang: "en".into(),
            target_lang: "de".into(),
            ..Default::default()
        }
    }

    // ── Content checks ────────────────────────────────────────────────────────

    #[test]
    fn prompt_contains_source_segment() {
        let p = build_prompt(&make_req());
        assert!(p.contains("Please sign the document."));
    }

    #[test]
    fn prompt_contains_source_and_target_langs() {
        let p = build_prompt(&make_req());
        assert!(p.contains("en"));
        assert!(p.contains("ja"));
    }

    #[test]
    fn prompt_contains_tm_score() {
        let p = build_prompt(&make_req());
        assert!(p.contains("82%"));
    }

    #[test]
    fn prompt_contains_tm_source_and_target() {
        let p = build_prompt(&make_req());
        assert!(p.contains("Sign the document"));
        assert!(p.contains("書類に署名する"));
    }

    #[test]
    fn prompt_contains_glossary_term() {
        let p = build_prompt(&make_req());
        assert!(p.contains("document"));
        assert!(p.contains("書類"));
    }

    #[test]
    fn prompt_ends_with_translation_label() {
        let p = build_prompt(&make_req());
        assert!(p.ends_with("TRANSLATION:\n"));
    }

    #[test]
    fn prompt_contains_domain_label() {
        let req = SuggestionRequest {
            source: "Sign here.".into(),
            source_lang: "en".into(),
            target_lang: "ja".into(),
            domain: Some("legal".into()),
            ..Default::default()
        };
        let p = build_prompt(&req);
        assert!(p.contains("legal domain"));
    }

    #[test]
    fn prompt_general_domain_when_none() {
        let p = build_prompt(&make_req());
        assert!(p.contains("general domain"));
    }

    #[test]
    fn prompt_includes_preceding_translation() {
        let req = SuggestionRequest {
            source: "Second sentence.".into(),
            source_lang: "en".into(),
            target_lang: "ja".into(),
            prev_target: Some("最初の文の翻訳。".into()),
            ..Default::default()
        };
        let p = build_prompt(&req);
        assert!(p.contains("PRECEDING TRANSLATION:"));
        assert!(p.contains("最初の文の翻訳。"));
    }

    #[test]
    fn prompt_no_preceding_translation_section_when_none() {
        let p = build_prompt(&make_req());
        assert!(!p.contains("PRECEDING TRANSLATION:"));
    }

    // ── Budget enforcement ────────────────────────────────────────────────────

    #[test]
    fn prompt_within_budget_normal_request() {
        let p = build_prompt(&make_req());
        assert!(p.len() <= 1800 + 50, "prompt is {} chars", p.len());
    }

    #[test]
    fn prompt_many_tm_matches_stays_within_budget() {
        let req = SuggestionRequest {
            source: "Source segment.".into(),
            source_lang: "en".into(),
            target_lang: "ja".into(),
            tm_matches: (0..50)
                .map(|i| TmContext {
                    source: format!("Source match number {i} with some extra words to consume budget"),
                    target: format!("ターゲット {i} 一致するテキスト"),
                    score: 70 + (i % 30) as u8,
                })
                .collect(),
            ..Default::default()
        };
        let p = build_prompt(&req);
        assert!(p.len() <= 1800 + 50, "prompt is {} chars", p.len());
    }

    #[test]
    fn prompt_many_glossary_hits_stays_within_budget() {
        let req = SuggestionRequest {
            source: "Source segment.".into(),
            source_lang: "en".into(),
            target_lang: "ja".into(),
            glossary_hits: (0..100)
                .map(|i| GlossaryContext {
                    source_term: format!("source term {i}"),
                    target_term: format!("ターゲット用語 {i}"),
                })
                .collect(),
            ..Default::default()
        };
        let p = build_prompt(&req);
        assert!(p.len() <= 1800 + 50, "prompt is {} chars", p.len());
    }

    // ── Empty context ─────────────────────────────────────────────────────────

    #[test]
    fn prompt_no_tm_no_glossary_still_valid() {
        let p = build_prompt(&empty_req());
        assert!(p.contains("Hello world."));
        assert!(p.ends_with("TRANSLATION:\n"));
        // Section headers should not appear when there are no entries
        assert!(!p.contains("TM MATCHES (source →"));
        assert!(!p.contains("GLOSSARY TERMS (use exactly)"));
    }

    #[test]
    fn prompt_empty_source_still_produces_valid_prompt() {
        let req = SuggestionRequest {
            source: String::new(),
            source_lang: "en".into(),
            target_lang: "ja".into(),
            ..Default::default()
        };
        let p = build_prompt(&req);
        assert!(p.ends_with("TRANSLATION:\n"));
    }

    // ── Workflow guard ────────────────────────────────────────────────────────

    #[test]
    fn prompt_with_high_tm_match_has_score_in_header() {
        let req = SuggestionRequest {
            source: "The contract must be signed.".into(),
            source_lang: "en".into(),
            target_lang: "ja".into(),
            tm_matches: vec![TmContext {
                source: "The contract must be signed".into(),
                target: "契約は署名されなければならない".into(),
                score: 95,
            }],
            ..Default::default()
        };
        let p = build_prompt(&req);
        assert!(p.contains("95%"));
        assert!(p.contains("契約は署名されなければならない"));
    }

    #[test]
    fn prompt_multiple_glossary_terms_all_included_when_space_permits() {
        let req = SuggestionRequest {
            source: "Sign the contract and submit the document.".into(),
            source_lang: "en".into(),
            target_lang: "ja".into(),
            glossary_hits: vec![
                GlossaryContext {
                    source_term: "contract".into(),
                    target_term: "契約".into(),
                },
                GlossaryContext {
                    source_term: "document".into(),
                    target_term: "書類".into(),
                },
            ],
            ..Default::default()
        };
        let p = build_prompt(&req);
        assert!(p.contains("契約"));
        assert!(p.contains("書類"));
    }
}
