use cheshire_qa::{QaIssue, run_checks};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct QaIssueDto {
    pub kind: String,
    pub message: String,
}

impl From<QaIssue> for QaIssueDto {
    fn from(i: QaIssue) -> Self {
        Self { kind: i.kind, message: i.message }
    }
}

/// Run QA checks on a source/target pair and return any issues found.
#[tauri::command]
pub fn qa_run(source: String, target: String) -> Vec<QaIssueDto> {
    run_checks(&source, &target)
        .into_iter()
        .map(QaIssueDto::from)
        .collect()
}
