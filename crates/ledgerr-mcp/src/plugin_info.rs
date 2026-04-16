//! plugin_info — version check, host metadata, decision log, and (Windows-only) self-update.
//!
//! The `plugin_info` MCP tool is always available and returns:
//!   - current embedded version
//!   - latest GitHub release version (when reachable)
//!   - whether an update is available
//!   - host system metadata
//!   - path to the decision/update log
//!
//! Subcommands (passed as the optional `subcommand` argument):
//!   - `"check"` (default) — version info + host metadata
//!   - `"upgrade"` — Windows-only; downloads and applies the latest release
//!   - `"cleanup"` — removes `*.old.exe` and `*.new.exe` backup files

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};

// ── Constants ─────────────────────────────────────────────────────────────────

pub const PLUGIN_INFO_TOOL: &str = "l3dg3rr_plugin_info";

const GITHUB_OWNER: &str = "PromptExecution";
const GITHUB_REPO: &str = "l3dg3rr";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

// ── Log path ──────────────────────────────────────────────────────────────────

/// Returns the path to the NDJSON decision log.
/// On Windows: `%APPDATA%\ledgerr-mcp\update.log`
/// On other platforms: `~/.local/share/ledgerr-mcp/update.log`
pub fn log_path() -> PathBuf {
    #[cfg(target_os = "windows")]
    let base = std::env::var("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(|_| std::env::temp_dir());
    #[cfg(not(target_os = "windows"))]
    let base = std::env::var("HOME")
        .map(|h| PathBuf::from(h).join(".local").join("share"))
        .unwrap_or_else(|_| std::env::temp_dir());

    base.join("ledgerr-mcp").join("update.log")
}

// ── Update log ────────────────────────────────────────────────────────────────

#[derive(serde::Serialize)]
struct LogEntry<'a> {
    ts: u64,
    pid: u32,
    event: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    detail: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    old_version: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    new_version: Option<&'a str>,
}

/// Append a single NDJSON entry to the log file.  Best-effort — never panics.
pub fn log_event(event: &str, detail: Option<&str>, old_ver: Option<&str>, new_ver: Option<&str>) {
    use std::io::Write as _;

    let path = log_path();
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    let entry = LogEntry {
        ts,
        pid: std::process::id(),
        event,
        detail,
        old_version: old_ver,
        new_version: new_ver,
    };
    let Ok(mut line) = serde_json::to_string(&entry) else {
        return;
    };
    line.push('\n');

    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&path) {
        let _ = f.write_all(line.as_bytes());
    }
}

// ── Host metadata ─────────────────────────────────────────────────────────────

/// Collect system metadata using `sysinfo` (cross-platform) plus
/// Windows registry extras via `winreg` when on Windows.
pub fn host_metadata() -> Value {
    let exe_path = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_default();

    // sysinfo provides OS name/version, uptime, CPU count without any unsafe code.
    let uptime_secs = sysinfo::System::uptime();
    let os_name = sysinfo::System::name().unwrap_or_default();
    let os_version = sysinfo::System::os_version().unwrap_or_default();
    let kernel_version = sysinfo::System::kernel_version().unwrap_or_default();
    let host_name = sysinfo::System::host_name().unwrap_or_default();
    let cpu_count = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(0);

    #[cfg_attr(not(target_os = "windows"), allow(unused_mut))]
    let mut meta = json!({
        "os": std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "family": std::env::consts::FAMILY,
        "exe_path": exe_path,
        "pid": std::process::id(),
        "os_name": os_name,
        "os_version": os_version,
        "kernel_version": kernel_version,
        "host_name": host_name,
        "cpu_logical_cores": cpu_count,
        "uptime_seconds": uptime_secs,
    });

    #[cfg(target_os = "windows")]
    windows_registry_extras(&mut meta);

    meta
}

/// Read additional Windows version strings from the registry via `winreg`.
#[cfg(target_os = "windows")]
fn windows_registry_extras(meta: &mut Value) {
    use winreg::enums::HKEY_LOCAL_MACHINE;
    use winreg::RegKey;

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let obj = meta.as_object_mut().expect("meta is object");

    if let Ok(key) = hklm.open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion") {
        let build: String = key.get_value("CurrentBuildNumber").unwrap_or_default();
        let product: String = key.get_value("ProductName").unwrap_or_default();
        let display: String = key.get_value("DisplayVersion").unwrap_or_default();
        obj.insert("windows_build".to_string(), json!(build));
        obj.insert("windows_product".to_string(), json!(product));
        obj.insert("windows_display_version".to_string(), json!(display));
    }
}

// ── GitHub release check ──────────────────────────────────────────────────────

pub struct ReleaseInfo {
    pub latest_tag: String,
    pub download_url: Option<String>,
}

/// Returns `(current, latest_tag, update_available, download_url_opt)`.
pub fn check_version() -> (String, String, bool, Option<String>) {
    let current = CURRENT_VERSION.to_string();

    #[cfg(feature = "self-update")]
    {
        if let Ok(info) = fetch_latest_release() {
            let latest_clean = info.latest_tag.trim_start_matches('v').to_string();
            let current_clean = current.trim_start_matches('v');
            let newer = is_newer(&latest_clean, current_clean);
            return (current, latest_clean, newer, info.download_url);
        }
    }

    (current, "unknown".to_string(), false, None)
}

#[cfg(feature = "self-update")]
fn fetch_latest_release() -> Result<ReleaseInfo, String> {
    let url = format!(
        "https://api.github.com/repos/{GITHUB_OWNER}/{GITHUB_REPO}/releases/latest"
    );
    let client = reqwest::blocking::Client::builder()
        .user_agent("ledgerr-mcp-self-update/1.0")
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let resp = client.get(&url).send().map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("GitHub API returned {}", resp.status()));
    }

    let body: Value = resp.json().map_err(|e| e.to_string())?;
    let tag = body["tag_name"]
        .as_str()
        .ok_or("missing tag_name")?
        .to_string();

    // Find the Windows exe asset.
    let download_url = body["assets"]
        .as_array()
        .and_then(|assets| {
            assets
                .iter()
                .find(|a| a["name"].as_str().map(|n| n.ends_with(".exe")).unwrap_or(false))
        })
        .and_then(|a| a["browser_download_url"].as_str())
        .map(|s| s.to_string());

    Ok(ReleaseInfo {
        latest_tag: tag,
        download_url,
    })
}

/// Returns true when `latest` semver is strictly greater than `current`.
/// Uses the `semver` crate when the `self-update` feature is active;
/// falls back to a plain tuple comparison otherwise.
pub fn is_newer(latest: &str, current: &str) -> bool {
    #[cfg(feature = "self-update")]
    {
        use semver::Version;
        match (Version::parse(latest), Version::parse(current)) {
            (Ok(l), Ok(c)) => l > c,
            _ => false,
        }
    }
    #[cfg(not(feature = "self-update"))]
    {
        parse_ver(latest) > parse_ver(current)
    }
}

#[cfg(not(feature = "self-update"))]
fn parse_ver(v: &str) -> (u64, u64, u64) {
    let mut parts = v.splitn(3, '.').map(|p| p.parse::<u64>().unwrap_or(0));
    (
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
        parts.next().unwrap_or(0),
    )
}

// ── Subcommand: cleanup ───────────────────────────────────────────────────────

/// Remove `*.old.exe` and `*.new.exe` sibling files next to the running binary.
pub fn cleanup_old_binaries() -> Vec<String> {
    let exe_dir = match std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
    {
        Some(d) => d,
        None => return vec!["error: could not determine exe directory".to_string()],
    };

    let Ok(entries) = std::fs::read_dir(&exe_dir) else {
        return vec!["error: could not read exe directory".to_string()];
    };

    let mut removed = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default();
        if name.contains(".old.exe") || name.ends_with(".new.exe") {
            match std::fs::remove_file(&path) {
                Ok(()) => {
                    log_event("cleanup_removed", Some(name), None, None);
                    removed.push(path.display().to_string());
                }
                Err(e) => removed.push(format!("error removing {}: {e}", path.display())),
            }
        }
    }
    removed
}

// ── Subcommand: upgrade (Windows + self-update feature only) ─────────────────

#[cfg(all(target_os = "windows", feature = "self-update"))]
pub fn perform_upgrade(download_url: &str, latest_version: &str) -> Value {
    use std::io::Write as _;

    log_event(
        "upgrade_start",
        Some(download_url),
        Some(CURRENT_VERSION),
        Some(latest_version),
    );

    let exe_path = match std::env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            log_event("upgrade_error", Some(&e.to_string()), None, None);
            return json!({ "error": "could not determine current exe path", "detail": e.to_string() });
        }
    };
    let exe_dir = exe_path.parent().expect("exe has parent");
    let new_path = exe_dir.join("ledgerr-mcp-server.new.exe");
    let old_path = exe_dir.join(format!("ledgerr-mcp-server-{CURRENT_VERSION}.old.exe"));

    // Step 1: download to .new.exe
    if let Err(e) = download_to(&new_path, download_url) {
        log_event("upgrade_download_error", Some(&e), None, None);
        return json!({ "error": "download failed", "detail": e });
    }
    log_event("upgrade_downloaded", Some(&new_path.display().to_string()), None, None);

    // Step 2: strip Zone.Identifier ADS (Mark of the Web) so SmartScreen won't block execution.
    strip_zone_identifier(&new_path);

    // Step 3: hash the file twice with a short pause — if they differ, AV quarantined it.
    let hash1 = blake3_file(&new_path);
    std::thread::sleep(std::time::Duration::from_millis(400));
    let hash2 = blake3_file(&new_path);
    if hash1 != hash2 || hash1.is_empty() {
        log_event(
            "upgrade_hash_mismatch",
            Some("file modified between hash checks — AV quarantine suspected"),
            None,
            None,
        );
        let _ = std::fs::remove_file(&new_path);
        return json!({
            "error": "hash instability detected — AV may have modified the download",
            "detail": "retry after allowlisting the binary"
        });
    }
    log_event("upgrade_hash_stable", Some(&hash1), None, None);

    // Step 4: rename current exe → versioned .old.exe (Windows allows rename of running exe).
    if let Err(e) = std::fs::rename(&exe_path, &old_path) {
        log_event("upgrade_rename_old_error", Some(&e.to_string()), None, None);
        let _ = std::fs::remove_file(&new_path);
        return json!({ "error": "could not rename current binary to .old.exe", "detail": e.to_string() });
    }
    log_event("upgrade_renamed_old", Some(&old_path.display().to_string()), None, None);

    // Step 5: rename .new.exe → current exe name.
    if let Err(e) = std::fs::rename(&new_path, &exe_path) {
        log_event("upgrade_rename_new_error", Some(&e.to_string()), None, None);
        // Best-effort rollback.
        let _ = std::fs::rename(&old_path, &exe_path);
        return json!({ "error": "could not promote .new.exe (rolled back)", "detail": e.to_string() });
    }
    log_event(
        "upgrade_complete",
        Some(&exe_path.display().to_string()),
        Some(CURRENT_VERSION),
        Some(latest_version),
    );

    json!({
        "status": "upgraded",
        "old_version": CURRENT_VERSION,
        "new_version": latest_version,
        "old_binary": old_path.display().to_string(),
        "note": "new binary is in place; restart the server to run the updated version"
    })
}

#[cfg(all(target_os = "windows", feature = "self-update"))]
fn download_to(dest: &std::path::Path, url: &str) -> Result<(), String> {
    use std::io::Write as _;

    let client = reqwest::blocking::Client::builder()
        .user_agent("ledgerr-mcp-self-update/1.0")
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| e.to_string())?;

    let mut resp = client.get(url).send().map_err(|e| e.to_string())?;
    if !resp.status().is_success() {
        return Err(format!("HTTP {}", resp.status()));
    }
    let mut f = std::fs::File::create(dest).map_err(|e| e.to_string())?;
    resp.copy_to(&mut f).map_err(|e| e.to_string())?;
    Ok(())
}

/// Delete `<path>:Zone.Identifier` alternate data stream added by Windows to
/// downloaded files.  Silently no-ops if the stream doesn't exist.
#[cfg(target_os = "windows")]
fn strip_zone_identifier(path: &std::path::Path) {
    let ads = format!("{}:Zone.Identifier", path.display());
    let _ = std::fs::remove_file(ads);
}

/// BLAKE3 hex digest of a file's contents; returns `""` on any I/O error.
#[cfg(all(target_os = "windows", feature = "self-update"))]
fn blake3_file(path: &std::path::Path) -> String {
    std::fs::read(path)
        .map(|bytes| blake3::hash(&bytes).to_hex().to_string())
        .unwrap_or_default()
}

// ── Main dispatcher ───────────────────────────────────────────────────────────

/// Entry point called from `mcp_adapter::handle_plugin_info`.
///
/// Accepts an optional `subcommand` field in `arguments`:
/// - `"check"` (default) — version + host metadata
/// - `"upgrade"` — self-update (Windows + `self-update` feature only)
/// - `"cleanup"` — remove `.old.exe` / `.new.exe` backup files
pub fn handle(arguments: &Value) -> Value {
    match arguments
        .get("subcommand")
        .and_then(Value::as_str)
        .unwrap_or("check")
    {
        "cleanup" => handle_cleanup(),
        "upgrade" => handle_upgrade(),
        _ => handle_check(),
    }
}

fn handle_check() -> Value {
    let (current, latest, update_available, _) = check_version();
    log_event("plugin_info_check", None, Some(&current), Some(&latest));

    json!({
        "current_version": current,
        "latest_version": latest,
        "update_available": update_available,
        "upgrade_hint": if update_available {
            format!("call {PLUGIN_INFO_TOOL} with subcommand='upgrade' to update")
        } else {
            "no update available".to_string()
        },
        "log_path": log_path().display().to_string(),
        "host": host_metadata(),
    })
}

fn handle_cleanup() -> Value {
    let removed = cleanup_old_binaries();
    json!({
        "removed": removed,
        "count": removed.len(),
        "log_path": log_path().display().to_string(),
    })
}

fn handle_upgrade() -> Value {
    #[cfg(all(target_os = "windows", feature = "self-update"))]
    {
        let (current, latest, update_available, download_url) = check_version();
        if !update_available {
            return json!({
                "status": "already_current",
                "version": current,
                "latest": latest,
            });
        }
        return match download_url {
            Some(url) => perform_upgrade(&url, &latest),
            None => json!({
                "status": "error",
                "error": "no Windows .exe asset found in latest release",
                "latest_tag": latest,
            }),
        };
    }

    #[cfg(not(all(target_os = "windows", feature = "self-update")))]
    json!({
        "status": "not_supported",
        "reason": "self-update requires a Windows build compiled with the self-update Cargo feature",
        "current_version": CURRENT_VERSION,
    })
}

// ── Tool schema ───────────────────────────────────────────────────────────────

pub fn input_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "subcommand": {
                "type": "string",
                "enum": ["check", "upgrade", "cleanup"],
                "description": "Action: 'check' (default) returns version info and host metadata; 'upgrade' downloads and installs the latest release (Windows + self-update feature only); 'cleanup' removes *.old.exe backup files."
            }
        },
        "required": []
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_newer_returns_true_when_latest_greater() {
        assert!(is_newer("1.4.0", "1.3.8"));
        assert!(is_newer("2.0.0", "1.99.99"));
        assert!(is_newer("1.3.9", "1.3.8"));
    }

    #[test]
    fn is_newer_returns_false_when_same_or_older() {
        assert!(!is_newer("1.3.8", "1.3.8"));
        assert!(!is_newer("1.3.7", "1.3.8"));
        assert!(!is_newer("0.1.0", "1.0.0"));
    }

    #[test]
    fn log_path_is_non_empty() {
        let p = log_path();
        assert!(p.to_str().map(|s| !s.is_empty()).unwrap_or(false));
        assert!(p.file_name().is_some());
    }

    #[test]
    fn log_event_does_not_panic() {
        log_event("test_event", Some("detail"), Some("0.1.0"), Some("0.2.0"));
    }

    #[test]
    fn host_metadata_has_required_fields() {
        let meta = host_metadata();
        assert!(meta["os"].is_string());
        assert!(meta["arch"].is_string());
        assert!(meta["pid"].is_number());
        assert!(meta["uptime_seconds"].is_number());
    }

    #[test]
    fn handle_check_returns_expected_shape() {
        let result = handle_check();
        assert!(result["current_version"].is_string());
        assert!(result["update_available"].is_boolean());
        assert!(result["log_path"].is_string());
        assert!(result["host"].is_object());
    }

    #[test]
    fn handle_cleanup_returns_array() {
        let result = handle_cleanup();
        assert!(result["removed"].is_array());
        assert!(result["count"].is_number());
    }

    #[test]
    fn handle_upgrade_not_supported_without_windows_feature() {
        #[cfg(not(all(target_os = "windows", feature = "self-update")))]
        {
            let result = handle_upgrade();
            assert_eq!(result["status"], "not_supported");
        }
    }

    #[test]
    fn input_schema_valid_shape() {
        let schema = input_schema();
        assert_eq!(schema["type"], "object");
        assert!(schema["properties"]["subcommand"]["enum"].is_array());
    }

    #[cfg(all(target_os = "windows", feature = "self-update"))]
    #[test]
    fn blake3_file_empty_for_missing_path() {
        assert_eq!(blake3_file(std::path::Path::new("/no/such/file.bin")), "");
    }

    #[test]
    fn cleanup_old_binaries_no_panic() {
        let _ = cleanup_old_binaries();
    }

    #[test]
    fn handle_dispatches_to_check_by_default() {
        let result = handle(&serde_json::Value::Object(Default::default()));
        assert!(result["current_version"].is_string());
    }
}
