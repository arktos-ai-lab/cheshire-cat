use cheshire_tm::{record::LangPair, Matcher};
use tauri::State;

use crate::{settings::Settings, state::AppState};

/// Normalise a BCP-47 language tag: replace `_` with `-`, lowercase the primary
/// subtag, uppercase 2-char region subtags, title-case 4-char script subtags.
/// e.g. `en_us` → `en-US`, `zh_Hans_CN` → `zh-Hans-CN`.
fn normalise_lang(tag: &str) -> String {
    if tag.is_empty() {
        return String::new();
    }
    let normalized = tag.replace('_', "-");
    let parts: Vec<&str> = normalized.split('-').map(str::trim).collect();
    parts
        .iter()
        .enumerate()
        .map(|(i, part)| {
            if i == 0 {
                part.to_lowercase()
            } else if part.len() == 4 {
                // Script subtag: title-case
                let mut c = part.chars();
                match c.next() {
                    None => String::new(),
                    Some(f) => f.to_uppercase().collect::<String>() + &c.as_str().to_lowercase(),
                }
            } else {
                part.to_uppercase()
            }
        })
        .collect::<Vec<_>>()
        .join("-")
}

#[tauri::command]
pub async fn settings_get(state: State<'_, AppState>) -> Result<Settings, String> {
    let mut s = state.settings.read().await.clone();
    s.is_windows = cfg!(target_os = "windows");
    Ok(s)
}

#[tauri::command]
pub async fn settings_set(
    mut new_settings: Settings,
    state: State<'_, AppState>,
) -> Result<(), String> {
    // Normalise language codes to proper BCP-47 form
    new_settings.source_lang = normalise_lang(&new_settings.source_lang);
    new_settings.target_lang = normalise_lang(&new_settings.target_lang);
    let json =
        serde_json::to_string_pretty(&new_settings).map_err(|e| e.to_string())?;

    std::fs::write(state.settings_path.as_ref(), &json).map_err(|e| e.to_string())?;

    // Rebuild the in-memory matcher if the language pair changed.
    let pair_changed = {
        let old = state.settings.read().await;
        old.source_lang != new_settings.source_lang || old.target_lang != new_settings.target_lang
    };

    *state.settings.write().await = new_settings.clone();

    if pair_changed && (!new_settings.source_lang.is_empty() || !new_settings.target_lang.is_empty()) {
        let lp = LangPair::new(&new_settings.source_lang, &new_settings.target_lang);
        let records = state
            .tm
            .load_for_matching(&lp)
            .await
            .unwrap_or_default();
        *state.matcher.write().await = Matcher::new(records);
    }

    Ok(())
}
