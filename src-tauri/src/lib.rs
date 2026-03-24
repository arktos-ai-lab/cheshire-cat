use std::{path::PathBuf, sync::Arc};

use cheshire_ai::{AiConfig, Orchestrator};
use cheshire_tm::{record::LangPair, Matcher, TmStore};
use tokio::sync::RwLock;

mod commands;
mod http_api;
mod settings;
mod state;

use settings::Settings;
use state::AppState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            use tauri::Manager;

            let data_dir = app
                .path()
                .app_data_dir()
                .expect("failed to resolve app data directory");

            std::fs::create_dir_all(&data_dir)
                .expect("failed to create app data directory");

            let settings_path = data_dir.join("settings.json");

            let settings: Settings = if settings_path.exists() {
                std::fs::read_to_string(&settings_path)
                    .ok()
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default()
            } else {
                Settings::default()
            };

            let tm_path: PathBuf = settings
                .tm_db_path
                .as_deref()
                .map(PathBuf::from)
                .unwrap_or_else(|| data_dir.join("tm.db"));

            let app_state = tauri::async_runtime::block_on(async {
                let tm = TmStore::open(&tm_path)
                    .await
                    .expect("failed to open TM database");

                let lang_pair =
                    LangPair::new(&settings.source_lang, &settings.target_lang);

                let records = tm
                    .load_for_matching(&lang_pair)
                    .await
                    .unwrap_or_default();

                let matcher = Arc::new(RwLock::new(Matcher::new(records)));
                let glossary = Arc::new(tm.glossary_store());

                // Build AI orchestrator from persisted settings
                let ai_config = AiConfig {
                    mode: settings.ai_mode,
                    base_url: settings.ai_url.clone(),
                    model: settings.ai_model.clone(),
                    api_key: if settings.ai_api_key.is_empty() {
                        None
                    } else {
                        Some(settings.ai_api_key.clone())
                    },
                    ..AiConfig::default()
                };
                let orchestrator = Arc::new(Orchestrator::new(ai_config));

                let settings_arc = Arc::new(RwLock::new(settings));

                // Spawn the Office.js add-in HTTP API as a background task
                let api_state = http_api::ApiState {
                    matcher: Arc::clone(&matcher),
                    glossary: Arc::clone(&glossary),
                    settings: Arc::clone(&settings_arc),
                };
                tauri::async_runtime::spawn(http_api::start(api_state, 8765));

                AppState {
                    tm: Arc::new(tm),
                    glossary,
                    matcher,
                    orchestrator,
                    settings: settings_arc,
                    settings_path: Arc::new(settings_path),
                }
            });

            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::tm::tm_search,
            commands::tm::tm_exact,
            commands::tm::tm_import,
            commands::tm::tm_export,
            commands::tm::tm_add_record,
            commands::tm::tm_delete_record,
            commands::tm::tm_count,
            commands::tm::tm_list,
            commands::glossary::glossary_lookup,
            commands::glossary::glossary_insert,
            commands::glossary::glossary_delete,
            commands::glossary::glossary_list_all,
            commands::glossary::glossary_count,
            commands::settings::settings_get,
            commands::settings::settings_set,
            commands::formats::format_import,
            commands::formats::format_export_xliff,
            commands::formats::format_export_po,
            commands::formats::format_export_json,
            commands::formats::format_export_csv,
            commands::formats::format_export_docx,
            commands::formats::format_export_bilingual_docx,
            commands::qa::qa_run,
            commands::ai::ai_suggest,
            commands::office::office_get_word_selection,
            commands::office::office_insert_into_word,
            commands::office::office_word_is_running,
            commands::office::office_get_excel_selection,
            commands::office::office_insert_into_excel,
            commands::office::office_excel_is_running,
            commands::office::office_get_ppt_selection,
            commands::office::office_insert_into_ppt,
            commands::office::office_ppt_is_running,
            commands::migration::migrate_from_felix2,
            commands::update::check_for_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Cheshire CAT");
}
