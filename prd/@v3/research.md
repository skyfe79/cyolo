# Research: Symlink-based Config Sharing (v3)

## Domain Overview

v3 adds the symlink layer that makes `cyolo profile add` useful beyond mere registration. When a new profile directory is created (e.g., `~/.claude-work`), cyolo should automatically symlink the 6 shared items from `~/.claude/` into it, so settings, skills, commands, and agents are shared across all profiles. The challenge is handling the many edge cases: missing sources, existing targets, the self-referencing case when `~/.claude` itself is registered, and ensuring the `profile link` re-creation command is idempotent.

## Key Technologies & Libraries

- **`std::os::unix::fs::symlink(original, link)`**: The stdlib function for creating Unix symlinks. It is type-agnostic — a single function handles both file and directory symlinks on Unix (unlike Windows which requires `symlink_file` / `symlink_dir`). Returns `io::Result<()>`. Fails with `AlreadyExists` if the link path already exists.

- **`std::fs::symlink_metadata(path)`**: Queries metadata about a path *without* following symlinks. Essential for detecting whether a target path is an existing symlink, regular file, or directory before attempting to create a symlink there. Maps to `lstat(2)` on Unix.

- **`std::fs::read_link(path)`**: Reads the target of an existing symlink. Useful for `profile link` to check whether an existing symlink already points to the correct source, enabling idempotent re-creation.

- **`std::path::Path::canonicalize()`**: Resolves symlinks and relative paths to an absolute path. Useful for comparing whether `~/.claude` is the same path as the profile's `config_dir` (the self-reference check).

- **No additional crates needed**: `std::os::unix::fs::symlink` covers the Unix case. The `symlink` crate on crates.io is only needed for cross-platform abstractions, which this project (macOS/Linux only) does not require.

## Architecture Recommendations

### Absolute vs. Relative Symlinks

**Decision: Use absolute symlinks.**

- Absolute symlinks always resolve correctly regardless of the working directory or how the symlink is accessed.
- Relative symlinks are better for portable/movable directory trees, but cyolo profiles live at fixed well-known paths (`~/.claude-<name>`), so portability is not a concern.
- GNU Stow uses relative symlinks because stow directories can be mounted at varying locations. That constraint does not apply here.

### Symlink Creation Pattern

```rust
use std::os::unix::fs::symlink;
use std::fs;
use std::path::Path;

fn create_shared_symlink(source: &Path, target: &Path) -> Result<(), SymlinkError> {
    // 1. Check source exists
    if !source.exists() {
        // For directories: create empty dir at source first, then symlink
        // For files: skip — creating an empty file could confuse Claude Code
        return Ok(());  // or warn
    }

    // 2. Check target does NOT exist (use symlink_metadata to catch broken symlinks)
    match fs::symlink_metadata(target) {
        Ok(_metadata) => {
            // Something exists at target — do not overwrite, warn user
            eprintln!("warning: {} already exists, skipping symlink", target.display());
            return Ok(());
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // Good — nothing at target path, proceed
        }
        Err(e) => return Err(e.into()),  // Permission error or other
    }

    // 3. Create symlink: target -> source
    symlink(source, target)?;
    Ok(())
}
```

### The 6 Shared Items

| Item | Type | If source missing | If target exists |
|------|------|-------------------|------------------|
| `CLAUDE.md` | file | skip symlink | warn, skip |
| `settings.json` | file | skip symlink | warn, skip |
| `settings.local.json` | file | skip symlink | warn, skip |
| `commands/` | directory | create empty dir at source, then symlink | warn, skip |
| `skills/` | directory | create empty dir at source, then symlink | warn, skip |
| `agents/` | directory | create empty dir at source, then symlink | warn, skip |

Rationale for directories: empty directories are harmless, and once they exist as a symlink target, future `npx` installs into `~/.claude/skills/` will be visible in all profiles immediately. Files (CLAUDE.md, settings.json) should NOT be created empty because Claude Code may misinterpret an empty settings file.

### Self-Reference Detection

When the user registers `~/.claude` itself as a profile (e.g., `cyolo profile add personal ~/.claude`), creating symlinks would be circular (symlink `~/.claude/CLAUDE.md` → `~/.claude/CLAUDE.md`). Detection:

```rust
fn is_source_dir(config_dir: &Path) -> bool {
    let source = dirs::home_dir().unwrap().join(".claude");
    // Use canonicalize to resolve any symlinks in the paths themselves
    match (source.canonicalize(), config_dir.canonicalize()) {
        (Ok(a), Ok(b)) => a == b,
        _ => {
            // If either doesn't exist yet, fall back to string comparison
            source == *config_dir
        }
    }
}
```

When detected: skip all symlink creation, print note: `Skipping symlinks: config_dir is the source (~/.claude)`.

### Integration with `profile add`

v3 modifies the existing `profile::add()` function (from v2) by inserting a symlink step between directory creation and config save:

```
1. Validate name not duplicate
2. Resolve config_dir
3. Create config_dir with 0700 (existing v2 logic)
4. [NEW] Create symlinks (unless --no-share or self-reference)
5. Register profile in config
6. Save config
```

The `--no-share` flag should be parsed in `profile::add()` args handling.

### `profile link` Command

New subcommand for re-creating symlinks on an already-registered profile:

```
cyolo profile link <name>
```

Behavior:
1. Load config, find profile by name
2. Run the same symlink creation logic as `profile add`
3. Idempotent: if symlink already exists and points to correct target, do nothing

This is useful when:
- Source items were added after profile creation (e.g., user created `~/.claude/CLAUDE.md` later)
- A symlink was accidentally deleted
- Upgrading from `--no-share` to shared

## Risks & Constraints

### TOCTOU Race Condition
Check-then-create for symlinks is technically racy (another process could create a file between check and symlink call). For a single-user CLI tool, this is acceptable. The `symlink()` call itself will fail with `AlreadyExists` if something appears, so the worst case is a confusing error message.

### Broken Symlinks
A symlink whose target has been deleted still appears to exist via `symlink_metadata()` but `Path::exists()` returns false (it follows the symlink). Always use `symlink_metadata()` when checking the link path — `exists()` would miss broken symlinks and incorrectly try to create a new symlink, failing with `AlreadyExists`.

### Canonicalize on Non-Existent Paths
`Path::canonicalize()` fails if the path doesn't exist. For the self-reference check, fall back to string comparison when canonicalize fails. This handles the edge case where `~/.claude` doesn't exist yet (fresh install).

### Permission Issues
Symlink creation requires write permission to the parent directory of the link. The profile directory was just created with 0700, so this should always succeed. Still, wrap in proper error handling with a descriptive message.

### Symlink to Symlink
If `~/.claude/skills/` is itself a symlink (common in some setups), `std::os::unix::fs::symlink` creates a symlink to the symlink — not to the final target. This is correct behavior for our use case: if the user updates the intermediate symlink, the profile symlinks follow the chain.

## Lessons from Previous Version (v2)

- **All 7 tasks completed successfully** in v2 with 0 failures. The module structure (main.rs, cli.rs, runner.rs, error.rs, config.rs, profile.rs) is solid and well-organized.
- **Manual argument routing** works well for the current complexity level. v3 adds `--no-share` flag to `profile add` and a new `link` subcommand — both fit naturally into the existing match-based dispatch.
- **Atomic config writes** established in v2 continue to work. v3 doesn't change config schema — symlinks are filesystem-level, not stored in config.json.
- **Error handling pattern** (CyoloError enum with descriptive variants) should be extended with symlink-specific variants for clear user messages.
- **expand_tilde() helper** in profile.rs is reusable for resolving the source path (~/.claude).

## Reference Links

- [std::os::unix::fs::symlink](https://doc.rust-lang.org/std/os/unix/fs/fn.symlink.html) - Rust stdlib Unix symlink creation
- [std::fs::symlink_metadata](https://doc.rust-lang.org/nightly/std/fs/fn.symlink_metadata.html) - Query metadata without following symlinks
- [std::fs::read_link](https://doc.rust-lang.org/std/fs/fn.read_link.html) - Read symlink target
- [GNU Stow Manual](https://www.gnu.org/software/stow/manual/stow.html) - Reference for symlink farm management patterns
- [claude-account-switcher](https://github.com/ukogan/claude-account-switcher) - Reference implementation of CLAUDE_CONFIG_DIR + symlink sharing
- [DotState](https://dotstate.serkan.dev) - Modern Rust dotfile manager with profile + symlink sharing patterns
- [Absolute vs Relative Symlinks](https://www.baeldung.com/linux/absolute-and-relative-symlinks) - Comparison for config management use cases
