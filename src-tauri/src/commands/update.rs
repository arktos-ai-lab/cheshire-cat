//! Version check against GitHub Releases.
//!
//! Compares `CARGO_PKG_VERSION` with the tag of the latest GitHub release.
//! Returns metadata so the frontend can show an update banner and open the
//! browser download page — no auto-download or auto-install.

use semver::Version;
use serde::Serialize;

/// Hard-coded GitHub repository for release checks.
/// Change this to your own org/repo before shipping.
const GITHUB_REPO: &str = "cheshire-cat/cheshire-cat";

/// Result returned to the frontend.
#[derive(Debug, Serialize)]
pub struct UpdateInfo {
    /// Whether a newer version is available on GitHub.
    pub available: bool,
    /// The current application version.
    pub current_version: String,
    /// The latest version string from GitHub (tag without leading `v`).
    pub latest_version: String,
    /// HTML URL of the latest GitHub release page.
    pub release_url: String,
    /// First paragraph of the release body (markdown).
    pub release_notes: String,
}

/// Check GitHub Releases for a newer version of Cheshire CAT.
///
/// Returns gracefully on network errors — `available` will be `false`.
#[tauri::command]
pub async fn check_for_update() -> Result<UpdateInfo, String> {
    let current = env!("CARGO_PKG_VERSION");

    let client = reqwest::Client::builder()
        .user_agent(format!("cheshire-cat/{current}"))
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!("https://api.github.com/repos/{GITHUB_REPO}/releases/latest");

    let resp = client
        .get(&url)
        .send()
        .await
        .map_err(|_| "Update check failed: could not reach GitHub".to_string())?;

    if !resp.status().is_success() {
        return Err(format!("GitHub API returned HTTP {}", resp.status()));
    }

    let json: serde_json::Value = resp.json().await.map_err(|e| e.to_string())?;

    let tag = json["tag_name"]
        .as_str()
        .unwrap_or("")
        .trim_start_matches('v');

    let release_url = json["html_url"]
        .as_str()
        .unwrap_or("")
        .to_string();

    let release_notes = json["body"]
        .as_str()
        .unwrap_or("")
        .lines()
        .next()
        .unwrap_or("")
        .to_string();

    // Semver comparison — treat any parse failure as "not newer"
    let available = Version::parse(tag)
        .ok()
        .zip(Version::parse(current).ok())
        .map(|(latest, cur)| latest > cur)
        .unwrap_or(false);

    Ok(UpdateInfo {
        available,
        current_version: current.to_string(),
        latest_version: tag.to_string(),
        release_url,
        release_notes,
    })
}
