use std::path::Path;

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
