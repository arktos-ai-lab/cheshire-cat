//! XLIFF 1.2 and 2.0 parser/writer.
//!
//! XLIFF is the standard bilingual file format used by agencies and most
//! modern CAT tools. Felix 2.0 did not support it — this is a critical addition.

// Placeholder — full implementation in Phase 3.
// The module is declared now so the crate compiles and the API surface
// is visible for integration work.

use std::path::Path;
use crate::{error::Result, record::{ImportStats, Record}};

/// Import records from an XLIFF 1.2 or 2.0 file.
/// Detects version from the root element.
pub async fn import(_path: &Path) -> Result<(Vec<Record>, ImportStats)> {
    // TODO Phase 3: implement XLIFF 1.2/2.0 parser
    Ok((Vec::new(), ImportStats::default()))
}

/// Export records to XLIFF 2.0.
pub async fn export(_path: &Path, _records: &[Record]) -> Result<()> {
    // TODO Phase 3
    Ok(())
}
