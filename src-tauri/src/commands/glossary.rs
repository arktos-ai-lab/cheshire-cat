use cheshire_tm::{GlossaryHit, GlossaryTerm};
use tauri::State;
use uuid::Uuid;

use crate::state::AppState;

#[tauri::command]
pub async fn glossary_lookup(
    text: String,
    state: State<'_, AppState>,
) -> Result<Vec<GlossaryHit>, String> {
    let settings = state.settings.read().await;
    let source_lang = settings.source_lang.clone();
    let target_lang = settings.target_lang.clone();
    drop(settings);

    state
        .glossary
        .lookup_in(&text, &source_lang, &target_lang)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn glossary_insert(
    source_term: String,
    target_term: String,
    domain: Option<String>,
    note: Option<String>,
    forbidden: bool,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let settings = state.settings.read().await;
    let source_lang = settings.source_lang.clone();
    let target_lang = settings.target_lang.clone();
    drop(settings);

    let mut term = GlossaryTerm::new(source_term, target_term, source_lang, target_lang);
    term.domain = domain;
    term.note = note;
    term.forbidden = forbidden;

    let id = state
        .glossary
        .insert(&term)
        .await
        .map_err(|e| e.to_string())?;

    Ok(id.to_string())
}

#[tauri::command]
pub async fn glossary_delete(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let uuid: Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    state
        .glossary
        .delete(uuid)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn glossary_list_all(state: State<'_, AppState>) -> Result<Vec<GlossaryTerm>, String> {
    let settings = state.settings.read().await;
    let source_lang = settings.source_lang.clone();
    let target_lang = settings.target_lang.clone();
    drop(settings);

    state
        .glossary
        .list_all(&source_lang, &target_lang)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn glossary_count(state: State<'_, AppState>) -> Result<i64, String> {
    state.glossary.count().await.map_err(|e| e.to_string())
}
