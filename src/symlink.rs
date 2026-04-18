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

    // --- Hermetic create_shared_symlinks_in tests (product PRD §4.6) ---
    //
    // All tests below drive the hermetic core against a TempDir-rooted
    // `source_base` so they never touch the real `~/.claude`.  Symlink
    // comparisons canonicalize both sides to survive the macOS
    // `/tmp` -> `/private/tmp` rewrite.

    #[cfg(unix)]
    #[test]
    fn create_in_links_existing_source_directory() {
        use tempfile::TempDir;

        let source_base = TempDir::new().unwrap();
        let target_base = TempDir::new().unwrap();

        // Pre-populate every shared item so the code path never has to
        // auto-create or warn — this test exercises the happy path only.
        for item in SHARED_ITEMS {
            let src = source_base.path().join(item.name);
            match item.kind {
                ItemKind::Directory => fs::create_dir_all(&src).unwrap(),
                ItemKind::File => fs::write(&src, b"stub").unwrap(),
            }
        }

        create_shared_symlinks_in(source_base.path(), target_base.path()).unwrap();

        let commands_link = target_base.path().join("commands");
        let resolved = fs::read_link(&commands_link).unwrap().canonicalize().unwrap();
        let expected = source_base.path().join("commands").canonicalize().unwrap();
        assert_eq!(resolved, expected);
    }

    #[cfg(unix)]
    #[test]
    fn create_in_auto_creates_missing_source_directory() {
        use tempfile::TempDir;

        let source_base = TempDir::new().unwrap();
        let target_base = TempDir::new().unwrap();
        // Pre-create the file-kind sources so the loop only has to auto-create
        // the directory-kind ones.  No dirs pre-exist in `source_base`.
        for item in SHARED_ITEMS {
            if item.kind == ItemKind::File {
                fs::write(source_base.path().join(item.name), b"stub").unwrap();
            }
        }

        create_shared_symlinks_in(source_base.path(), target_base.path()).unwrap();

        // Directory-kind sources should have been auto-created (§4.6 rule 1).
        let created_source = source_base.path().join("commands");
        assert!(created_source.is_dir(), "source dir should be auto-created");

        let link = target_base.path().join("commands").canonicalize().unwrap();
        let expected = created_source.canonicalize().unwrap();
        assert_eq!(link, expected);
    }

    #[cfg(unix)]
    #[test]
    fn create_in_skips_missing_source_file() {
        use tempfile::TempDir;

        let source_base = TempDir::new().unwrap();
        let target_base = TempDir::new().unwrap();
        // Only populate directory-kind sources — every file-kind source is
        // intentionally missing so §4.6 rule 2 (skip-and-warn) is exercised.
        for item in SHARED_ITEMS {
            if item.kind == ItemKind::Directory {
                fs::create_dir_all(source_base.path().join(item.name)).unwrap();
            }
        }

        create_shared_symlinks_in(source_base.path(), target_base.path()).unwrap();

        for item in SHARED_ITEMS {
            let target = target_base.path().join(item.name);
            match item.kind {
                ItemKind::File => {
                    assert!(
                        fs::symlink_metadata(&target).is_err(),
                        "file-kind target should not be created when source is missing: {}",
                        item.name
                    );
                }
                ItemKind::Directory => {
                    assert!(
                        fs::symlink_metadata(&target).is_ok(),
                        "dir-kind target should still be linked: {}",
                        item.name
                    );
                }
            }
        }
    }

    #[cfg(unix)]
    #[test]
    fn create_in_skips_preexisting_target() {
        use tempfile::TempDir;

        let source_base = TempDir::new().unwrap();
        let target_base = TempDir::new().unwrap();
        for item in SHARED_ITEMS {
            let src = source_base.path().join(item.name);
            match item.kind {
                ItemKind::Directory => fs::create_dir_all(&src).unwrap(),
                ItemKind::File => fs::write(&src, b"stub").unwrap(),
            }
        }

        // Pre-existing regular file at one target must not be overwritten.
        let preexisting = target_base.path().join("CLAUDE.md");
        fs::write(&preexisting, b"pre-existing content").unwrap();

        create_shared_symlinks_in(source_base.path(), target_base.path()).unwrap();

        let meta = fs::symlink_metadata(&preexisting).unwrap();
        assert!(
            meta.file_type().is_file(),
            "pre-existing regular file must not be replaced by a symlink (§4.6 rule 3)"
        );
        assert_eq!(fs::read(&preexisting).unwrap(), b"pre-existing content");
    }

    #[cfg(unix)]
    #[test]
    fn create_in_skips_when_config_dir_is_source() {
        use tempfile::TempDir;

        let source_base = TempDir::new().unwrap();
        for item in SHARED_ITEMS {
            let src = source_base.path().join(item.name);
            match item.kind {
                ItemKind::Directory => fs::create_dir_all(&src).unwrap(),
                ItemKind::File => fs::write(&src, b"stub").unwrap(),
            }
        }

        // Snapshot children before the call so we can assert nothing changed.
        let mut before: Vec<_> = fs::read_dir(source_base.path())
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        before.sort();

        // config_dir == source_base → §5.1 early-return, no symlinks touched.
        create_shared_symlinks_in(source_base.path(), source_base.path()).unwrap();

        let mut after: Vec<_> = fs::read_dir(source_base.path())
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        after.sort();
        assert_eq!(before, after, "no new entries should be created when source == target");

        // Sanity: none of the entries should be a symlink (all were seeded as
        // regular files/dirs above).
        for item in SHARED_ITEMS {
            let meta = fs::symlink_metadata(source_base.path().join(item.name)).unwrap();
            assert!(!meta.file_type().is_symlink(), "{} became a symlink", item.name);
        }
    }
}
