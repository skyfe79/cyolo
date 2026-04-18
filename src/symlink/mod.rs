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

/// Returns `true` when `config_dir` and `source` resolve to the same
/// directory.  Tries `canonicalize` first; falls back to a plain path
/// comparison when either side does not yet exist on disk.
fn is_source_dir_in(config_dir: &Path, source: &Path) -> bool {
    if let (Ok(a), Ok(b)) = (config_dir.canonicalize(), source.canonicalize()) {
        return a == b;
    }
    config_dir == source
}

/// Returns `true` when `config_dir` resolves to `~/.claude` (the source
/// directory itself).  Symlink creation should be skipped in that case to
/// avoid circular self-references.
pub fn is_source_dir(config_dir: &Path) -> bool {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return false,
    };
    is_source_dir_in(config_dir, &home.join(".claude"))
}

/// Creates symlinks for all [`SHARED_ITEMS`] from `~/.claude/` into `config_dir`.
///
/// Follows a warn-and-continue strategy: if any single item fails (missing
/// source file, existing target, symlink I/O error) it prints a message to
/// stderr and moves on to the next item.
pub fn create_shared_symlinks(config_dir: &Path) -> Result<(), CyoloError> {
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
    create_shared_symlinks_in(&home.join(".claude"), config_dir)
}

/// Hermetic core of [`create_shared_symlinks`] that accepts an explicit
/// `source_base` so tests can drive the behavior against a `TempDir` instead
/// of the real `~/.claude`.
fn create_shared_symlinks_in(source_base: &Path, config_dir: &Path) -> Result<(), CyoloError> {
    if is_source_dir_in(config_dir, source_base) {
        eprintln!(
            "{} config dir is ~/.claude itself, skipping symlink creation",
            "warning:".yellow().bold()
        );
        return Ok(());
    }

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
mod tests;
