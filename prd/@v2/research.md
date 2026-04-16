# Research: Global Config & Profile Registry (v2)

## Domain Overview

v2 adds the configuration layer that underpins all profile operations. It creates the `~/.cyolo/` directory with secure permissions and implements a JSON-based profile registry (`config.json`). Operations include reading/writing the registry atomically, adding profiles (with directory creation), removing profiles (config-only, preserving the directory), and listing profiles with a default marker. No symlink or walk-up detection logic yet — those come in v3/v4.

## Key Technologies & Libraries

- **serde + serde_json 1.x** (already in Cargo.toml): Define `CyoloConfig` and `Profile` structs with `#[derive(Serialize, Deserialize)]`. Use `serde_json::from_reader` / `serde_json::to_writer_pretty` for human-readable config. Use `#[serde(default)]` on optional fields to handle missing keys gracefully when loading older config files.

- **dirs 5.x** (already in Cargo.toml): `dirs::home_dir()` returns `Option<PathBuf>` for resolving `~`. Use this to construct `~/.cyolo/` and default profile paths like `~/.claude-<name>`. Never hardcode `/Users/...`.

- **std::os::unix::fs::DirBuilderExt**: For creating `~/.cyolo/` with mode `0o700` atomically at creation time. Use `DirBuilder::new().mode(0o700).create(path)` rather than creating then chmod, to avoid a window where the directory has wrong permissions.

- **std::fs + std::io**: For atomic config writes. Pattern: write to a temp file in the same directory, `flush()`, then `rename()` to the target. This prevents corruption if the process is killed mid-write. The `tempfile` crate is overkill for v2 — stdlib is sufficient since we control the temp file naming and cleanup.

- **thiserror 2.x** (already in Cargo.toml): Extend `CyoloError` with config-specific variants (config not found, parse failure, IO errors, profile already exists, profile not found).

## Architecture Recommendations

### Config Schema

```rust
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CyoloConfig {
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub profiles: BTreeMap<String, Profile>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Profile {
    pub name: String,
    pub config_dir: PathBuf,
}
```

Use `BTreeMap` over `HashMap` for deterministic key ordering in the JSON output — makes diffs and manual edits predictable.

### Module Structure (v2 additions)

```
src/
├── main.rs       # Entry point (unchanged)
├── cli.rs        # Route profile subcommands to profile module
├── runner.rs     # Process execution (unchanged)
├── error.rs      # Add config/profile error variants
├── config.rs     # CyoloConfig struct, load/save, atomic write
└── profile.rs    # Profile CRUD: add, rm, list, dispatch
```

Keep `config.rs` focused on the data layer (schema + IO) and `profile.rs` on the command layer (argument parsing + user-facing output). This separation keeps config reusable for diet commands in v6.

### Atomic Write Pattern (stdlib-only)

```rust
use std::fs;
use std::io::Write;
use std::path::Path;

fn atomic_write(path: &Path, content: &[u8]) -> std::io::Result<()> {
    let tmp_path = path.with_extension("tmp");
    let mut file = fs::File::create(&tmp_path)?;
    file.write_all(content)?;
    file.flush()?;
    file.sync_all()?;   // fsync — ensures data reaches disk
    fs::rename(&tmp_path, path)?;
    Ok(())
}
```

Key points:
- Temp file must be on the **same filesystem** as target for `rename()` to be atomic. Using `.with_extension("tmp")` in the same directory guarantees this.
- `sync_all()` is the Rust equivalent of `fsync()` — without it, data may sit in kernel buffers during a crash.
- If the process dies before `rename()`, the `.tmp` file is left behind but the original config is intact. Next startup can clean up stale `.tmp` files.

### Profile Add Flow

```
1. Load config (or create default if missing)
2. Validate: name not already registered
3. Resolve config_dir: if omitted, use ~/.claude-<name>
4. Create config_dir with 0o700 if it doesn't exist
5. Insert Profile into config.profiles
6. If --default flag: set config.default = Some(name)
7. Atomic-write config.json
8. Print confirmation
```

### Profile Rm Flow

```
1. Load config
2. Validate: name exists in profiles
3. Remove from config.profiles
4. If config.default == Some(name): clear default
5. Atomic-write config.json
6. Print note: "Directory preserved. Remove manually: rm -rf <path>"
```

### Profile List Output Format

```
* personal → /Users/codingmax/.claude
  work     → /Users/codingmax/.claude-work
```

Left-align profile names, pad to longest name width. `*` prefix marks the default. If no profiles registered, print: "No profiles registered. Run: cyolo profile add <name>"

### CLI Routing Update

v1's `cli.rs` returns `Err(NotImplemented("profile"))` for profile commands. v2 replaces this with actual dispatch:

```rust
Command::Profile(args) => profile::dispatch(&args),
```

Parse profile sub-subcommands manually: `add`, `rm`, `list`. Other profile commands (default, init, current, link) come in v4/v5.

## Risks & Constraints

- **Race condition on config write**: If two cyolo processes modify config.json simultaneously, one write will be lost. Acceptable for single-user CLI — document as known limitation. File locking (flock) can be added in v9 if needed.

- **tilde expansion in config_dir**: Users may pass `~/.claude-work` on the command line. Shell typically expands `~` before it reaches the binary, but if it doesn't (e.g., inside quotes), cyolo must handle it. Use `dirs::home_dir()` to manually expand leading `~`.

- **Directory permissions on existing directories**: `DirBuilder::new().mode(0o700).create()` fails if the directory already exists. Use `create_dir_all` equivalent or check existence first. `profile add` with an existing directory should succeed (reuse it), not error.

- **Config file doesn't exist on first run**: `config::load()` must distinguish "file not found" (return default empty config) from "file exists but has parse error" (return error). Do not silently create config.json until the user actually runs a command that needs it.

- **Path display**: Always use `PathBuf::display()` for user-facing output. Store paths as `PathBuf` in the struct, serialize as strings in JSON using `#[serde(with = "path_serde")]` or let serde handle it naturally (serde_json serializes PathBuf as string on Unix).

## Lessons from Previous Version (v1)

- **Manual arg routing works well**: v1's `match args.first()` pattern for top-level routing is clean and avoids clap fighting with pass-through args. v2 should use the same manual approach for profile sub-subcommands (`add`, `rm`, `list`) since clap integration for `profile` will come later when more complex flags are needed (v5).

- **Error type extension is straightforward**: v1's `CyoloError` enum with `thiserror` derives is easy to extend. Add new variants for config/profile errors without touching existing ones.

- **Module boundaries are clean**: v1's `main.rs → cli.rs → runner.rs` chain is well-separated. v2 adds `config.rs` and `profile.rs` as siblings, not nested — keeps the dependency graph flat.

- **All v1 tasks completed without issues**: The Cargo project structure, error types, runner, CLI routing, and integration verification all passed. No technical debt to address.

## Reference Links

- [atomic-write-file crate](https://docs.rs/atomic-write-file) - Rust crate for atomic file writes (reference, not used — stdlib suffices)
- [Rust forum: atomic file writes](https://users.rust-lang.org/t/how-to-write-replace-files-atomically/42821) - Discussion of tempfile+fsync+rename pattern
- [DirBuilder docs](https://doc.rust-lang.org/std/fs/struct.DirBuilder.html) - Rust stdlib directory creation with mode
- [PermissionsExt trait](https://doc.rust-lang.org/std/os/unix/fs/trait.PermissionsExt.html) - Unix permission bits in Rust
- [claude-account-switcher](https://github.com/ukogan/claude-account-switcher) - Reference: symlink-based profile sharing for Claude Code
- [claude-switch (Rust)](https://lib.rs/crates/claude-switch) - Rust implementation of Claude account switching
- [Rain's Rust CLI recommendations](https://rust-cli-recommendations.sunshowers.io/configuration.html) - Best practices for CLI config management
- [confy crate](https://github.com/rust-cli/confy) - Zero-boilerplate config management (reference for patterns, not used)
