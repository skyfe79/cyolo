# cyolo

Multi-account profile manager and config cleaner for Claude Code.

## Motivation

- You juggle multiple Claude Code accounts (personal, work, clients) and
  switching means logging out, logging in, or remembering to export
  `CLAUDE_CONFIG_DIR` every shell.
- Each new profile directory would otherwise need its own copy of the things
  you set up once — `CLAUDE.md`, `settings.json`, `commands/`, `skills/`,
  `agents/` — so npm-installed skills and shared prompts drift out of sync.
- `~/.claude.json` grows without bound. Deleted projects stay in the
  `projects` map; `~/.claude/projects/<encoded-path>/` keeps their session
  history; multi-megabyte configs slow Claude Code startup for no reason.

`cyolo` solves all three: per-folder profile auto-detection, symlink-based
sharing of the six common config items, and a `diet` command that reports
and (on `--apply`) reclaims the orphaned data.

## Installation

`cyolo` is a single Rust binary. From a clone of this repository, either use
the install script (a thin wrapper that runs the cargo command below and
reports on `~/.cargo/bin` PATH status):

```bash
./install.sh            # release build
./install.sh --debug    # dev build (faster, unoptimized)
./install.sh --locked   # pin to Cargo.lock (CI-friendly)
```

or invoke cargo directly:

```bash
cargo install --path .
```

Both drop the binary into `~/.cargo/bin/cyolo`. `~/.cyolo/` is used only for
config and profile directories, not the binary.

**MSRV: Rust 1.85+ is required** (the crate uses `edition = "2024"`). Check
with `rustc --version`; upgrade via `rustup update stable` if needed.

Once published to crates.io the installation will also work as:

```bash
cargo install cyolo   # future; not yet published
```

If you were previously using the `cyolo()` zsh function, remove it from your
shell rc file before using the binary — the binary replaces the function and
supports the same pass-through semantics.

## Quickstart

```bash
cyolo profile add personal ~/.claude             # register the existing ~/.claude as a profile (no symlinks — it is the source)
cyolo profile default personal                   # make it the fallback when no .claude-profile.json is found
cyolo profile add work                           # creates ~/.claude-work/ + symlinks shared config + launches `claude` so you can /login as the work account
cyolo profile list                               # shows "* personal  skyfe79@gmail.com" and "  work  work@example.com"
cyolo                                            # runs `claude --dangerously-skip-permissions` with the resolved profile
```

In a work project, drop a profile marker so every invocation from that tree
uses the right account:

```bash
cd ~/work/client-a && cyolo profile init work
cyolo                                            # resolves "work" via walk-up from anywhere beneath ~/work/client-a
```

## How multi-account OAuth actually works

Claude Code stores its OAuth token in the macOS Keychain. The service name
is composed dynamically from `CLAUDE_CONFIG_DIR`:

```
CLAUDE_CONFIG_DIR unset        → Claude Code-credentials
CLAUDE_CONFIG_DIR=~/.claude-work → Claude Code-credentials-<sha256("/Users/you/.claude-work")[:8]>
```

Because each profile directory hashes to a different suffix, **each profile
gets its own distinct Keychain entry**. Two Anthropic accounts can coexist —
no re-login when you switch profiles, no overwritten tokens. cyolo simply
sets `CLAUDE_CONFIG_DIR` before launching `claude` and lets Claude Code
itself pick the right Keychain entry.

The account identity (email, organization, subscription tier) is stored in
`<CLAUDE_CONFIG_DIR>/.claude.json` under `oauthAccount`. `cyolo profile list`
and `cyolo profile whoami` read this file to show you which account a profile
is currently bound to.

### Two-account tutorial

```bash
cyolo profile add personal ~/.claude         # register existing login as "personal" (no extra login needed)
cyolo profile default personal
cyolo profile add work                       # creates ~/.claude-work/, auto-opens `claude` → run /login with your second Anthropic account
cyolo profile list                           # both profiles listed with their emails
cd ~/work/project && cyolo profile init work # bind this tree to the work profile
cyolo                                        # from inside ~/work/... → work account; elsewhere → personal
```

If you skip the auto-login (`--no-login` on `cyolo profile add`), you can
always run `cyolo profile login <name>` later.

## Usage — profile subcommands

Nine subcommands cover the full profile lifecycle.

### add

```bash
cyolo profile add <name> [config-dir] [--no-share] [--no-login]
```

Register a new profile. `config-dir` defaults to `~/.claude-<name>`.
Missing directories are created with `0700`. The six shared items are
symlinked from `~/.claude/` unless `--no-share` is given. Registering
`~/.claude` itself creates no symlinks (it is the source). To mark the
new profile as the default, run `cyolo profile default <name>`
afterward.

Immediately after registration, `cyolo` launches `claude` with the new
`CLAUDE_CONFIG_DIR` so you can run `/login` and bind the intended Anthropic
account to this profile's Keychain entry. Pass `--no-login` to skip the
launch (useful when you are re-registering a profile that already has a
valid token, or when running in CI).

```bash
cyolo profile add client ~/.claude-client-a
cyolo profile add scratch --no-login          # register without spawning claude
```

### login

```bash
cyolo profile login <name>
```

Re-run the interactive login flow for a registered profile. Useful when
a refresh token expires or when you want to swap the profile to a
different Anthropic account. Equivalent to the launch that `add` does by
default.

### whoami

```bash
cyolo profile whoami
```

Like `current`, but also prints the `oauthAccount.emailAddress` extracted
from the resolved profile's `.claude.json`. If the profile has never been
logged in, the email line reads `(needs login — run cyolo profile login <name>)`.

### rm

```bash
cyolo profile rm <name>
```

Remove a profile from `~/.cyolo/config.json`. The on-disk directory is
preserved — delete it yourself with `rm -rf ~/.claude-<name>` if needed.

### list

```bash
cyolo profile list
```

Tabulate all registered profiles. The default is marked `*`.

```
* personal → /Users/codingmax/.claude
  work     → /Users/codingmax/.claude-work
  client   → /Users/codingmax/.claude-client-a
```

### default

```bash
cyolo profile default [name | --unset]
```

With no arguments, prints the current default. Given a registered name,
sets it. `--unset` clears the default (no fallback during resolution).

```bash
cyolo profile default work
cyolo profile default --unset
```

### init

```bash
cyolo profile init [name]
```

Write `.claude-profile.json` into the current directory so walk-up detection
resolves to `name` from this tree. If `name` is omitted, the current default
is used. Refuses to overwrite an existing file.

```bash
cyolo profile init work
```

### current

```bash
cyolo profile current
```

Print the profile that would be used by `cyolo` right now (walk-up →
default → unset). Does not launch `claude`.

```
profile: work
config_dir: /Users/codingmax/.claude-work
source: /Users/codingmax/work/client-a/.claude-profile.json
```

### link

```bash
cyolo profile link <name>
```

Idempotently (re)create the six shared symlinks for an already-registered
profile. Use this after adding a new shared item in `~/.claude/` or if a
symlink is broken.

## Usage — diet

`diet` reports and reclaims orphaned Claude Code data. By default it is
read-only.

```bash
cyolo diet                              # dry-run report, current profile
cyolo diet --apply                      # actually remove orphaned entries + session folders
cyolo diet --stale-days 90              # include projects idle ≥ 90 days (dry-run)
cyolo diet --stale-days 90 --apply      # remove orphaned + prune stale history
cyolo diet --cache                      # include cache dirs (statsig, shell-snapshots, file-history)
cyolo diet --profile work               # operate on a specific registered profile
cyolo diet --all                        # iterate every registered profile
```

Sample dry-run report (tree format, matches the spec):

```
$ cyolo diet
cyolo diet — analyzing /Users/codingmax/.claude

~/.claude.json:                          1.2 MB  (6840 lines)
  ├─ orphaned projects (5):              980 KB  (removable)
  │   ├─ /Users/codingmax/Private/labs/test-bot      320 KB
  │   ├─ /Users/codingmax/tmp/experiment             210 KB
  │   └─ ... 3 more
  └─ active configuration:               220 KB  (keep)

~/.claude/projects/:                      847 MB
  └─ orphaned session folders (5):       623 MB  (removable)

Total reclaimable: 624 MB

Run with --apply to proceed.
```

Safety: `--apply` automatically writes a timestamped backup
(`~/.claude.json.backup-<YYYYMMDDHHMMSS>`) and aborts if a `claude`
process is already running.

## How it works

### Walk-up resolution

At every invocation `cyolo` searches the current directory upward for
`.claude-profile.json`, the same way `git` finds `.git`. The first file
found wins. Without one, the default profile is used; without a default,
`CLAUDE_CONFIG_DIR` is left unset and Claude Code falls back to
`~/.claude` (matching the original `cyolo()` shell function exactly).

### Symlink-based sharing

Each non-source profile directory (everything except `~/.claude` itself)
is a plain directory with six symlinks back into `~/.claude/`:

```
CLAUDE.md             → ~/.claude/CLAUDE.md
settings.json         → ~/.claude/settings.json
settings.local.json   → ~/.claude/settings.local.json
commands/             → ~/.claude/commands
skills/               → ~/.claude/skills
agents/               → ~/.claude/agents
```

Install a skill once (`cd ~/.claude/skills && npx install-some-skill`) and
every profile sees it. Credentials, session history, and runtime caches
stay per-profile — they are never symlinked.

### Diet orphan detection

`diet` reads `~/.claude.json`, iterates the `projects` map, and flags
every key whose filesystem path no longer exists. For each orphan it also
locates the matching session folder under `~/.claude/projects/` (the key
is path-encoded) and sums its size. `--apply` removes both the JSON
entries and the session folders, atomically rewriting the config via a
temp file + `rename`.

## Building from source

```bash
cargo build --release        # binary at target/release/cyolo
cargo test                   # unit tests (all modules)
```

There are no external build tools or codegen steps — a plain `cargo build`
is sufficient.

## Platform support

macOS and Linux only. Windows is **not supported**: the symlink model,
`0700` permission enforcement, and `pgrep`-based running-process
detection all assume POSIX semantics. A WSL or MSYS2 environment may
work but is untested and unsupported.

## License

MIT — see [LICENSE](LICENSE) for the full text.
