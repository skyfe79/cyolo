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
mod tests;
