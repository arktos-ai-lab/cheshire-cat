use std::{path::PathBuf, sync::Arc};
use tokio::sync::RwLock;

use cheshire_ai::Orchestrator;
use cheshire_tm::{GlossaryStore, Matcher, TmStore};

use crate::settings::Settings;

/// Shared application state, managed by Tauri.
///
/// All fields are wrapped in `Arc` so they can be cheaply cloned for the
/// background HTTP API server without duplicating the underlying data.
pub struct AppState {
    pub tm: Arc<TmStore>,
    pub glossary: Arc<GlossaryStore>,
    pub matcher: Arc<RwLock<Matcher>>,
    pub orchestrator: Arc<Orchestrator>,
    pub settings: Arc<RwLock<Settings>>,
    /// Path to the settings JSON file so commands can persist changes.
    pub settings_path: Arc<PathBuf>,
}
