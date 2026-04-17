#!/usr/bin/env python3
"""Task 006: Wire dispatch() with profile loop, stale detection, cache handling, and single atomic write"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-006",
    version="v8",
    title="Wire dispatch() with profile loop, stale/cache handling, clear_stale_history, and single atomic write",
    description="""Full pipeline integration in dispatch(). This is the most complex task:

1. Implement clear_stale_history() — in-memory only mutation of parsed_json to empty
   history arrays for stale projects. Does not write to disk.

2. Refactor remove_orphaned_entries() — currently both mutates JSON AND writes to disk
   (line 568). Must split: keep in-memory mutation, remove the disk write. The single
   atomic_write_json() call happens in dispatch() after both orphan removal and stale
   history clearing.

3. Refactor apply() — currently orchestrates backup + orphan removal + session removal.
   Must be updated or replaced since the dispatch flow now handles per-profile operations
   and a single JSON write.

4. Update dispatch() with the full pipeline:
   - parse args → safety check → resolve profiles → parse ~/.claude.json once
   - FOR each profile: analyze orphans, detect stale, measure cache, scan sessions, build report
   - IF apply (once, after loop): backup, rotate, mutate JSON (orphans + stale), single write,
     remove orphaned sessions, remove stale sessions, remove cache contents

Key constraint: ~/.claude.json is always at $HOME/.claude.json regardless of profile.
With --all, orphan cleanup and stale history clearing happen ONCE on the shared JSON,
while cache and session cleanup happen per-profile.""",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="high",
    estimated_hours=6,
    phase="Phase 4: Integration",
    dependencies=["task-001", "task-002", "task-003", "task-004", "task-005"],
    subtasks=[
        Subtask("task-006-1", "Implement clear_stale_history() in-memory mutation", 1),
        Subtask("task-006-2", "Refactor remove_orphaned_entries() to separate mutation from disk write", 1),
        Subtask("task-006-3", "Rewrite dispatch() with profile loop and single write strategy", 3),
        Subtask("task-006-4", "Write unit tests for clear_stale_history and integration tests", 1),
    ],
    acceptance_criteria=[
        "clear_stale_history mutates parsed_json in memory without writing to disk",
        "clear_stale_history empties history arrays for stale project paths",
        "clear_stale_history skips entries where history is not an array (warns to stderr)",
        "clear_stale_history preserves project entries (only clears history)",
        "remove_orphaned_entries no longer writes to disk (caller is responsible)",
        "dispatch() calls atomic_write_json exactly once after all mutations",
        "cyolo diet --stale-days 90 shows stale projects in dry-run",
        "cyolo diet --stale-days 90 --apply clears history + removes stale sessions",
        "cyolo diet --cache shows cache sizes in dry-run",
        "cyolo diet --cache --apply removes cache contents, preserves dirs",
        "cyolo diet --profile work targets only the work profile",
        "cyolo diet --all iterates all registered profiles",
        "Per-profile cache and session cleanup run per-profile",
        "Multi-profile output includes profile header lines",
        "cargo build succeeds",
        "cargo test passes",
    ],
    files=["src/diet.rs"],
    technical_notes="""CRITICAL: remove_orphaned_entries() at line 568 currently calls atomic_write_json
internally. This must be refactored so it only mutates parsed_json (the HashSet retain
logic stays), and the caller handles the write. This is a breaking change to that function's
contract — update all call sites.

The apply() function at line 658 currently calls remove_orphaned_entries (which writes).
Either inline its logic into dispatch() or refactor it to not write.

clear_stale_history signature:
  fn clear_stale_history(parsed_json: &mut Value, stale_paths: &[String])

For each stale path: parsed_json["projects"][path]["history"] = Value::Array(vec![])
Check that "history" exists and is_array() before clearing. If not array, warn + skip.

dispatch() flow (from PRD F6):
  1. parse_diet_args
  2. safety check (claude running)
  3. resolve_target_profiles
  4. parse ~/.claude.json once
  5. FOR each profile: orphan analysis, stale detection, cache measurement, session scan, report
  6. IF apply: backup, rotate, mutate JSON (orphans + stale), atomic_write_json, per-profile cleanup

When printing multiple profiles, add header: "--- profile: work (~/.claude-work) ---"

The atomic_write_json function (line 520) is currently private. It needs to stay accessible
from dispatch() — it's already in the same module so this is fine.""",
    web_search=[],
)

workflow = TaskWorkflow(task)

@workflow.pre_job
def load_context():
    """Load project context and skill files."""
    workflow.retrieve_memory("learning")
    workflow.read_skills()

@workflow.step(1)
def investigate():
    """
    Read src/diet.rs thoroughly:
    1. dispatch() (line 733) — current pipeline flow
    2. apply() (line 658) — current cleanup orchestration
    3. remove_orphaned_entries() (line 568) — mutation + write coupling
    4. atomic_write_json() (line 520) — write helper
    5. All new functions from tasks 1-5: detect_stale_projects, measure_cache_dirs,
       remove_cache_contents, resolve_target_profiles, updated DietReport/build_report_string
    """
    pass

@workflow.step(2)
def implement_clear_stale_history():
    """
    Implement clear_stale_history(parsed_json: &mut Value, stale_paths: &[String]):
    1. For each path in stale_paths:
       a. Access parsed_json["projects"][path]
       b. If entry exists and has "history" key:
          - If history.is_array(): replace with empty array
          - Else: eprintln warning, skip
       c. If entry doesn't exist: skip silently
    2. No disk write — pure in-memory mutation
    """
    pass

@workflow.step(3)
def refactor_remove_orphaned_entries():
    """
    Split remove_orphaned_entries into mutation-only:
    1. Remove the atomic_write_json call and serialization from remove_orphaned_entries
    2. Change signature: remove claude_json_path parameter (no longer needed)
       New: fn remove_orphaned_entries(parsed_json: &mut Value, orphaned_paths: &[String])
       Returns nothing (or keep Result for consistency but no IO errors possible)
    3. Keep the HashSet + retain logic unchanged
    4. Update all call sites (apply function, tests)
    5. Update tests that verify on-disk behavior to test in-memory mutation instead
    """
    pass

@workflow.step(4)
def rewrite_dispatch():
    """
    Rewrite dispatch() with full v8 pipeline:
    1. parse_diet_args → DietOptions
    2. if !force: is_claude_running check
    3. resolve_target_profiles(options)
    4. home_dir, claude_json_path = home.join(".claude.json")
    5. parse ~/.claude.json once → AnalysisResult
    6. Init accumulators: all_stale_paths, all_orphaned_sessions
    7. FOR each (name, claude_home) in profiles:
       - projects_dir = claude_home.join("projects")
       - Compute orphans for this profile (filter from shared analysis)
       - IF stale_days: detect_stale_projects → accumulate paths
       - IF cache: measure_cache_dirs
       - scan_session_folders
       - Build DietReport (with stale + cache) → print
       - IF multi-profile: print header before report
       - IF apply: remove_cache_contents, remove stale session folders (per-profile)
    8. IF apply (after loop):
       - backup_claude_json
       - rotate_backups(keep=5)
       - remove_orphaned_entries(parsed_json, all_orphan_paths) [in-memory]
       - clear_stale_history(parsed_json, all_stale_paths) [in-memory]
       - serialize + atomic_write_json [single disk write]
       - remove_session_folders(all_orphaned_sessions)
    """
    pass

@workflow.step(5)
def add_tests():
    """
    Write unit tests:
    1. test_clear_stale_history_basic: clears history array for matching paths
    2. test_clear_stale_history_preserves_entry: project entry preserved, only history emptied
    3. test_clear_stale_history_non_array: history is string → warned, skipped
    4. test_clear_stale_history_missing_path: path not in JSON → no error
    5. test_clear_stale_history_no_history_key: project has no history field → no error
    6. test_remove_orphaned_entries_no_write: verify mutation only (no file path needed)
    7. Update existing remove_orphaned_entries tests for new signature
    8. test_dispatch_unknown_arg: existing test still passes
    """
    pass

@workflow.step(6)
def codex_review_and_apply():
    """
    Run Codex AI code review and apply feedback:
    1. Run workflow.codex_review() to get review results
    2. Analyze the review feedback (especially around the refactored write strategy)
    3. Apply necessary fixes
    4. Re-run cargo build && cargo test
    """
    workflow.codex_review()

@workflow.verify
def check_builds():
    """Project builds without errors."""
    workflow.run_command("cargo build")

@workflow.verify
def check_tests_pass():
    """All tests pass."""
    workflow.run_command("cargo test")

@workflow.post_job
def save_learnings():
    """Record learnings."""
    workflow.update_memory("learning", "v8-task-006", "Wired full v8 diet pipeline with single atomic write strategy")

@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()

if __name__ == "__main__":
    workflow.run_cli()
