//! Carry the user-level MCP server configuration across cyolo profiles.
//!
//! Claude Code's "User MCPs" live in a single JSON file keyed at
//! `<CLAUDE_CONFIG_DIR>/.claude.json` (or `~/.claude.json` when the env
//! var is unset — see PRD §10.1). Spinning up a fresh profile therefore
//! produces a directory whose `.claude.json` contains only that profile's
//! OAuth + project history and **no** `mcpServers`, which is why the
//! MCP list looks empty inside any newly minted profile.
//!
//! This module copies just the `mcpServers` key out of the canonical
//! source (`~/.claude.json` at `$HOME`) into the target profile's
//! `.claude.json`, leaving every other key — `oauthAccount`, `projects`,
//! analytics caches, and all the rest — untouched so multi-account
//! isolation stays intact. The merge is atomic (temp + rename) and
//! permissions are tightened to `0o600` to match Claude Code's own
//! posture.

use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::error::CyoloError;

/// Canonical location of the user-level MCP config: `$HOME/.claude.json`.
///
/// This is the file Claude Code reads when `CLAUDE_CONFIG_DIR` is unset.
/// When the env is set, claude reads `<env>/.claude.json` instead — that
/// second file is the one we seed here.
pub fn source_path() -> Result<PathBuf, CyoloError> {
    dirs::home_dir()
        .map(|h| h.join(".claude.json"))
        .ok_or_else(|| CyoloError::ConfigIoError {
            context: "could not determine home directory".into(),
            source: std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "home directory not found",
            ),
        })
}

/// Read `mcpServers` from the given source file path.
///
/// Returns `Ok(None)` when the file is missing, unreadable, unparseable,
/// or simply does not contain a `mcpServers` key. The caller should treat
/// any `None` as "nothing to sync" rather than an error — a fresh install
/// legitimately lacks the file.
pub(crate) fn read_source_mcp_servers_from(path: &Path) -> Result<Option<Value>, CyoloError> {
    let bytes = match fs::read(path) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => {
            return Err(CyoloError::ConfigIoError {
                context: format!("failed to read {}", path.display()),
                source: e,
            });
        }
    };
    // An unreadable/malformed file should not abort cyolo workflows — the
    // user may have a transient edit in progress. Log-worthy, but best-effort.
    let Ok(parsed) = serde_json::from_slice::<Value>(&bytes) else {
        return Ok(None);
    };
    Ok(parsed.get("mcpServers").cloned())
}

/// Pure merge: take the existing target-file value (or any JSON shape —
/// missing file, empty, non-object) and produce a result where the top-level
/// object carries the given `mcp_servers` under the `mcpServers` key.
///
/// If the target is not a JSON object we overwrite it with a fresh object;
/// partially-corrupt files are upgraded to "just the mcpServers we know
/// about" rather than lost. This mirrors how cyolo treats the profile as
/// a scratchpad — claude is the canonical writer of every other key.
pub(crate) fn merge_mcp_servers(target: Value, mcp_servers: Value) -> Value {
    let mut obj = match target {
        Value::Object(map) => map,
        _ => serde_json::Map::new(),
    };
    obj.insert("mcpServers".to_string(), mcp_servers);
    Value::Object(obj)
}

/// Sync `mcpServers` from `~/.claude.json` into `<config_dir>/.claude.json`.
///
/// Returns the number of MCP entries written. `Ok(0)` covers "nothing to
/// do" (source missing, source has no `mcpServers`, or the object is
/// empty) — callers can treat 0 as a no-op without further inspection.
/// The write is atomic: temp file in the same directory, `fsync`, rename,
/// then `chmod 0600`. Any pre-existing target keys are preserved verbatim.
pub fn sync_mcp_to_profile(config_dir: &Path) -> Result<usize, CyoloError> {
    let source = source_path()?;
    sync_mcp_to_profile_from(&source, config_dir)
}

/// Like `sync_mcp_to_profile` but with an explicit source path.
///
/// Kept `pub(crate)` so tests can supply a temp-dir source without mutating
/// the process-wide `$HOME` env var (which would race across parallel
/// tests). Production callers always go through `sync_mcp_to_profile`.
pub(crate) fn sync_mcp_to_profile_from(
    source: &Path,
    config_dir: &Path,
) -> Result<usize, CyoloError> {
    let Some(source_mcp) = read_source_mcp_servers_from(source)? else {
        return Ok(0);
    };

    let count = source_mcp.as_object().map(|o| o.len()).unwrap_or(0);
    if count == 0 {
        return Ok(0);
    }

    fs::create_dir_all(config_dir).map_err(|e| CyoloError::ConfigIoError {
        context: format!("failed to ensure {}", config_dir.display()),
        source: e,
    })?;
    let target_path = config_dir.join(".claude.json");

    // Best-effort read of the existing target. Missing and malformed files
    // both degrade to an empty object; the caller's contract is "merge in
    // mcpServers, do not destroy other keys when present."
    let existing: Value = match fs::read(&target_path) {
        Ok(bytes) => serde_json::from_slice(&bytes).unwrap_or(Value::Object(Default::default())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Value::Object(Default::default()),
        Err(e) => {
            return Err(CyoloError::ConfigIoError {
                context: format!("failed to read {}", target_path.display()),
                source: e,
            });
        }
    };

    let merged = merge_mcp_servers(existing, source_mcp);
    let pretty = serde_json::to_string_pretty(&merged).map_err(|e| CyoloError::ConfigIoError {
        context: format!("failed to serialize {}", target_path.display()),
        source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
    })?;

    let tmp_path = target_path.with_extension("json.tmp");
    let mut file = fs::File::create(&tmp_path).map_err(|e| CyoloError::ConfigIoError {
        context: format!("failed to create temp file {}", tmp_path.display()),
        source: e,
    })?;
    file.write_all(pretty.as_bytes())
        .map_err(|e| CyoloError::ConfigIoError {
            context: format!("failed to write temp file {}", tmp_path.display()),
            source: e,
        })?;
    file.write_all(b"\n")
        .map_err(|e| CyoloError::ConfigIoError {
            context: format!("failed to write temp file {}", tmp_path.display()),
            source: e,
        })?;
    file.sync_all().map_err(|e| CyoloError::ConfigIoError {
        context: format!("failed to sync temp file {}", tmp_path.display()),
        source: e,
    })?;
    // Tighten perms before the rename so the file never appears with
    // umask-derived perms even for a moment.
    fs::set_permissions(&tmp_path, fs::Permissions::from_mode(0o600)).map_err(|e| {
        CyoloError::ConfigIoError {
            context: format!("failed to chmod temp file {}", tmp_path.display()),
            source: e,
        }
    })?;
    fs::rename(&tmp_path, &target_path).map_err(|e| CyoloError::ConfigIoError {
        context: format!(
            "failed to rename {} to {}",
            tmp_path.display(),
            target_path.display()
        ),
        source: e,
    })?;

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    #[test]
    fn merge_inserts_into_empty_object() {
        let merged = merge_mcp_servers(json!({}), json!({"agent": {}}));
        assert_eq!(merged, json!({"mcpServers": {"agent": {}}}));
    }

    #[test]
    fn merge_preserves_unrelated_keys() {
        let target = json!({"oauthAccount": {"emailAddress": "user@example.com"}, "projects": {}});
        let merged = merge_mcp_servers(target, json!({"agent": {}}));
        assert_eq!(
            merged,
            json!({
                "oauthAccount": {"emailAddress": "user@example.com"},
                "projects": {},
                "mcpServers": {"agent": {}}
            })
        );
    }

    #[test]
    fn merge_overrides_existing_mcp_servers() {
        // Contract: the source wins. `~/.claude.json` is the single source of
        // truth for user MCPs; a profile's older `mcpServers` entry is stale.
        let target = json!({"mcpServers": {"old": {}}});
        let merged = merge_mcp_servers(target, json!({"agent": {}}));
        assert_eq!(merged, json!({"mcpServers": {"agent": {}}}));
    }

    #[test]
    fn merge_replaces_non_object_target_root() {
        // Corrupt or stub `.claude.json` (e.g. an array) should not tank the
        // sync — we degrade to a fresh object carrying just mcpServers.
        let merged = merge_mcp_servers(json!([1, 2, 3]), json!({"agent": {}}));
        assert_eq!(merged, json!({"mcpServers": {"agent": {}}}));
    }

    // Each sync test lives in its own TempDir and passes the source path
    // explicitly — no $HOME mutation, no shared-state races under parallel
    // test execution.

    #[test]
    fn sync_creates_file_when_absent() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("source.json");
        fs::write(
            &source,
            r#"{"mcpServers": {"agent": {"url": "x"}, "github": {"url": "y"}}}"#,
        )
        .unwrap();

        let profile = tmp.path().join("profile-a");
        let written = sync_mcp_to_profile_from(&source, &profile).unwrap();
        assert_eq!(written, 2);

        let parsed: Value =
            serde_json::from_str(&fs::read_to_string(profile.join(".claude.json")).unwrap())
                .unwrap();
        assert_eq!(
            parsed["mcpServers"]["agent"]["url"],
            Value::String("x".into())
        );
        assert_eq!(
            parsed["mcpServers"]["github"]["url"],
            Value::String("y".into())
        );
    }

    #[test]
    fn sync_preserves_oauth_and_projects() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("source.json");
        fs::write(&source, r#"{"mcpServers": {"agent": {"url": "x"}}}"#).unwrap();

        let profile = tmp.path().join("profile-b");
        fs::create_dir_all(&profile).unwrap();
        fs::write(
            profile.join(".claude.json"),
            r#"{"oauthAccount": {"emailAddress": "x@y"}, "projects": {"/foo": {}}}"#,
        )
        .unwrap();

        assert_eq!(sync_mcp_to_profile_from(&source, &profile).unwrap(), 1);

        let parsed: Value =
            serde_json::from_str(&fs::read_to_string(profile.join(".claude.json")).unwrap())
                .unwrap();
        assert_eq!(
            parsed["oauthAccount"]["emailAddress"],
            Value::String("x@y".into())
        );
        assert!(parsed["projects"].as_object().unwrap().contains_key("/foo"));
        assert!(parsed["mcpServers"].as_object().unwrap().contains_key("agent"));
    }

    #[test]
    fn sync_no_op_when_source_missing() {
        let tmp = TempDir::new().unwrap();
        let profile = tmp.path().join("profile-c");
        assert_eq!(
            sync_mcp_to_profile_from(&tmp.path().join("absent.json"), &profile).unwrap(),
            0
        );
        assert!(!profile.join(".claude.json").exists());
    }

    #[test]
    fn sync_no_op_when_source_has_no_mcp_servers() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("source.json");
        fs::write(&source, r#"{"unrelated": true}"#).unwrap();

        let profile = tmp.path().join("profile-d");
        assert_eq!(sync_mcp_to_profile_from(&source, &profile).unwrap(), 0);
        assert!(!profile.join(".claude.json").exists());
    }

    #[test]
    fn sync_no_op_when_source_mcp_servers_is_empty() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("source.json");
        fs::write(&source, r#"{"mcpServers": {}}"#).unwrap();

        let profile = tmp.path().join("profile-e");
        assert_eq!(sync_mcp_to_profile_from(&source, &profile).unwrap(), 0);
        assert!(!profile.join(".claude.json").exists());
    }

    #[test]
    fn sync_overrides_stale_target_mcp_but_keeps_other_keys() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("source.json");
        fs::write(&source, r#"{"mcpServers": {"agent": {}}}"#).unwrap();

        let profile = tmp.path().join("profile-g");
        fs::create_dir_all(&profile).unwrap();
        fs::write(
            profile.join(".claude.json"),
            r#"{"oauthAccount": {"emailAddress": "x@y"}, "mcpServers": {"stale": {}}}"#,
        )
        .unwrap();

        assert_eq!(sync_mcp_to_profile_from(&source, &profile).unwrap(), 1);

        let parsed: Value =
            serde_json::from_str(&fs::read_to_string(profile.join(".claude.json")).unwrap())
                .unwrap();
        // Source won — "stale" is gone, "agent" is present.
        assert!(parsed["mcpServers"].as_object().unwrap().contains_key("agent"));
        assert!(
            !parsed["mcpServers"].as_object().unwrap().contains_key("stale"),
            "expected stale entry to be evicted by source override"
        );
        // OAuth is untouched.
        assert_eq!(
            parsed["oauthAccount"]["emailAddress"],
            Value::String("x@y".into())
        );
    }

    #[cfg(unix)]
    #[test]
    fn sync_applies_0600_permissions() {
        let tmp = TempDir::new().unwrap();
        let source = tmp.path().join("source.json");
        fs::write(&source, r#"{"mcpServers": {"agent": {"url": "x"}}}"#).unwrap();

        let profile = tmp.path().join("profile-f");
        sync_mcp_to_profile_from(&source, &profile).unwrap();

        let mode = fs::metadata(profile.join(".claude.json"))
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(
            mode & 0o777,
            0o600,
            "expected 0o600, got {:o}",
            mode & 0o777
        );
    }
}
