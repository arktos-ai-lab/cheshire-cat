use thiserror::Error;

pub mod orchestrator;
pub mod prompt;

pub use orchestrator::{
    AiConfig, AiMode, GlossaryContext, Orchestrator, Suggestion, SuggestionRequest, TmContext,
};

#[derive(Debug, Error)]
pub enum AiError {
    #[error("HTTP request failed: {0}")]
    Request(String),
    #[error("Failed to parse AI response: {0}")]
    Parse(String),
}

pub type Result<T> = std::result::Result<T, AiError>;
