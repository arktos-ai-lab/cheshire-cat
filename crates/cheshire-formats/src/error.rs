#[derive(Debug, thiserror::Error)]
pub enum FormatError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("XML parse error: {0}")]
    Xml(#[from] quick_xml::Error),

    #[error("ZIP error: {0}")]
    Zip(#[from] zip::result::ZipError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("unsupported file extension: {ext}")]
    UnknownExtension { ext: String },

    #[error("malformed {format} file: {reason}")]
    Malformed { format: &'static str, reason: String },
}

pub type Result<T> = std::result::Result<T, FormatError>;
