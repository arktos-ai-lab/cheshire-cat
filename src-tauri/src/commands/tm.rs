use cheshire_tm::{record::LangPair, ImportStats, Matcher, Record, TmMatch};
use tauri::State;
use uuid::Uuid;

use crate::state::AppState;

#[tauri::command]
pub async fn tm_search(
    query: String,
    state: State<'_, AppState>,
) -> Result<Vec<TmMatch>, String> {
    let settings = state.settings.read().await;
    let threshold = settings.fuzzy_threshold as f32 / 100.0;
    let max = settings.max_matches;
    drop(settings);

    let matcher = state.matcher.read().await;
    Ok(matcher.search(&query, threshold, max))
}

#[tauri::command]
pub async fn tm_exact(
    source: String,
    state: State<'_, AppState>,
) -> Result<Vec<Record>, String> {
    let settings = state.settings.read().await;
    let lp = LangPair::new(&settings.source_lang, &settings.target_lang);
    drop(settings);

    state
        .tm
        .exact_matches(&source, &lp)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tm_import(
    path: String,
    state: State<'_, AppState>,
) -> Result<ImportStats, String> {
    let p = std::path::Path::new(&path);
    let ext = p.extension().and_then(|e| e.to_str()).unwrap_or("").to_lowercase();
    let stats = match ext.as_str() {
        "xliff" | "xlf" => state.tm.import_xliff(p).await.map_err(|e| e.to_string())?,
        _ => state.tm.import_tmx(p).await.map_err(|e| e.to_string())?,
    };

    // Rebuild the in-memory matcher so new records are immediately searchable.
    let settings = state.settings.read().await;
    let lp = LangPair::new(&settings.source_lang, &settings.target_lang);
    drop(settings);

    let records = state
        .tm
        .load_for_matching(&lp)
        .await
        .map_err(|e| e.to_string())?;

    *state.matcher.write().await = Matcher::new(records);

    Ok(stats)
}

#[tauri::command]
pub async fn tm_export(
    path: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let settings = state.settings.read().await;
    let lp = LangPair::new(&settings.source_lang, &settings.target_lang);
    drop(settings);

    state
        .tm
        .export_tmx(std::path::Path::new(&path), Some(&lp))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tm_add_record(
    source: String,
    target: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let settings = state.settings.read().await;
    let source_lang = settings.source_lang.clone();
    let target_lang = settings.target_lang.clone();
    drop(settings);

    let record = Record::new(source, target, source_lang, target_lang);
    let id = state.tm.insert(&record).await.map_err(|e| e.to_string())?;

    // Update the live matcher without a full reload.
    state.matcher.write().await.add(record);

    Ok(id.to_string())
}

#[tauri::command]
pub async fn tm_delete_record(
    id: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let uuid: Uuid = id.parse().map_err(|e: uuid::Error| e.to_string())?;
    state.tm.delete(uuid).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tm_count(state: State<'_, AppState>) -> Result<i64, String> {
    state.tm.count().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn tm_list(
    limit: i64,
    offset: i64,
    state: State<'_, AppState>,
) -> Result<Vec<Record>, String> {
    let settings = state.settings.read().await;
    let lp = LangPair::new(&settings.source_lang, &settings.target_lang);
    drop(settings);

    state
        .tm
        .list_records(&lp, limit, offset)
        .await
        .map_err(|e| e.to_string())
}
