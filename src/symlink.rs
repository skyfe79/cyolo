use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use owo_colors::OwoColorize;

use crate::error::CyoloError;

/// Whether a shared item is a file or a directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemKind {
    File,
    Directory,
}

/// A shared configuration item that gets symlinked from `~/.claude/` into a
/// profile directory.
#[derive(Debug, Clone, Copy)]
pub struct SharedItem {
    pub name: &'static str,
    pub kind: ItemKind,
}

/// The six items shared between profiles via symlinks.
pub const SHARED_ITEMS: &[SharedItem] = &[
    SharedItem { name: "CLAUDE.md", kind: ItemKind::File },
    SharedItem { name: "settings.json", kind: ItemKind::File },
    SharedItem { name: "settings.local.json", kind: ItemKind::File },
    SharedItem { name: "commands", kind: ItemKind::Directory },
    SharedItem { name: "skills", kind: ItemKind::Directory },
    SharedItem { name: "agents", kind: ItemKind::Directory },
];

/// Returns `true` when `config_dir` resolves to `~/.claude` (the source
/// directory itself).  Symlink creation should be skipped in that case to
/// avoid circular self-references.
///
/// Tries `canonicalize` first so that symlinks and `..` components are
/// resolved.  Falls back to a plain path comparison when the target does not
/// exist on disk yet.
pub fn is_source_dir(config_dir: &Path) -> bool {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return false,
    };
    let source = home.join(".claude");

    // Prefer canonicalize (resolves symlinks and relative segments).
    if let (Ok(a), Ok(b)) = (config_dir.canonicalize(), source.canonicalize()) {
        return a == b;
    }

    // Fallback: direct comparison when paths may not exist yet.
    config_dir == source
}

/// Creates symlinks for all [`SHARED_ITEMS`] from `~/.claude/` into `config_dir`.
///
/// Follows a warn-and-continue strategy: if any single item fails (missing
/// source file, existing target, symlink I/O error) it prints a message to
/// stderr and moves on to the next item.
pub fn create_shared_symlinks(config_dir: &Path) -> Result<(), CyoloError> {
    if is_source_dir(config_dir) {
        eprintln!(
            "{} config dir is ~/.claude itself, skipping symlink creation",
            "warning:".yellow().bold()
        );
        return Ok(());
    }

    let home = match dirs::home_dir() {
        Some(h) => h,
        None => {
            eprintln!(
                "{} could not determine home directory, skipping symlink creation",
                "warning:".yellow().bold()
            );
            return Ok(());
        }
    };
    let source_base = home.join(".claude");

    for item in SHARED_ITEMS {
        let source = source_base.join(item.name);
        let target = config_dir.join(item.name);

        // Ensure source exists; create empty dirs, skip missing files.
        if !source.exists() {
            match item.kind {
                ItemKind::Directory => {
                    if let Err(e) = fs::create_dir_all(&source) {
                        eprintln!(
                            "{} failed to create source directory {}: {}",
                            "error:".red().bold(),
                            source.display(),
                            e
                        );
                        continue;
                    }
                    if let Err(e) = fs::set_permissions(&source, fs::Permissions::from_mode(0o755)) {
                        eprintln!(
                            "{} failed to set permissions on {}: {}",
                            "error:".red().bold(),
                            source.display(),
                            e
                        );
                        continue;
                    }
                }
                ItemKind::File => {
                    eprintln!(
                        "{} source file {} not found, skipping",
                        "warning:".yellow().bold(),
                        source.display()
                    );
                    continue;
                }
            }
        }

        // Detect existing target (symlink_metadata catches broken symlinks too).
        if fs::symlink_metadata(&target).is_ok() {
            eprintln!(
                "{} target {} already exists, skipping",
                "warning:".yellow().bold(),
                target.display()
            );
            continue;
        }

        // Create the symlink (absolute paths).
        if let Err(e) = std::os::unix::fs::symlink(&source, &target) {
            eprintln!(
                "{} failed to symlink {} -> {}: {}",
                "error:".red().bold(),
                source.display(),
                target.display(),
                e
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shared_items_has_six_entries() {
        assert_eq!(SHARED_ITEMS.len(), 6);
    }

    #[test]
    fn shared_items_file_kinds() {
        let files: Vec<&str> = SHARED_ITEMS
            .iter()
            .filter(|i| i.kind == ItemKind::File)
            .map(|i| i.name)
            .collect();
        assert_eq!(files, vec!["CLAUDE.md", "settings.json", "settings.local.json"]);
    }

    #[test]
    fn shared_items_directory_kinds() {
        let dirs: Vec<&str> = SHARED_ITEMS
            .iter()
            .filter(|i| i.kind == ItemKind::Directory)
            .map(|i| i.name)
            .collect();
        assert_eq!(dirs, vec!["commands", "skills", "agents"]);
    }

    #[test]
    fn is_source_dir_detects_home_claude() {
        if let Some(home) = dirs::home_dir() {
            assert!(is_source_dir(&home.join(".claude")));
        }
    }

    #[test]
    fn is_source_dir_rejects_other_paths() {
        assert!(!is_source_dir(std::path::Path::new("/tmp/some-profile")));
    }
}
