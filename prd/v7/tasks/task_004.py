#!/usr/bin/env python3
"""Task 004: Implement rotate_backups() and wire into apply()"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-004",
    version="v7",
    title="Implement rotate_backups() and wire into apply()",
    description="""Create rotate_backups(path, keep) that lists backup files matching
    <filename>.backup-* in the same directory, sorts alphabetically (YYYYMMDDHHMMSS sorts
    chronologically), keeps the latest N, and deletes the rest. Individual delete failures
    warn to stderr but don't abort. Call rotate_backups() in apply() after backup_claude_json().""",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="medium",
    estimated_hours=1.5,
    phase="Phase 2: Atomic Operations",
    dependencies=[],
    subtasks=[
        Subtask("task-004-1", "Implement rotate_backups(path, keep) -> Result<(), CyoloError>", 0.75),
        Subtask("task-004-2", "Call rotate_backups() in apply() after backup_claude_json()", 0.25),
        Subtask("task-004-3", "Unit tests with temp dir containing mock backup files", 0.5),
    ],
    acceptance_criteria=[
        "rotate_backups() exists in src/diet.rs",
        "When 6+ backups exist, oldest are deleted leaving exactly 5",
        "When <= 5 backups exist, none are deleted",
        "Individual delete failures warn to stderr but don't abort",
        "apply() calls rotate_backups(&claude_json_path, 5) after backup",
        "cargo test passes with temp dir containing mock backup files",
    ],
    files=["src/diet.rs"],
    technical_notes="""The backup files follow the pattern: <filename>.backup-YYYYMMDDHHMMSS
    (e.g., claude.json.backup-20260417123456). Since format_timestamp produces YYYYMMDDHHMMSS,
    alphabetical sort = chronological sort. Use fs::read_dir to list files, filter by prefix,
    sort, then fs::remove_file for excess. The path parameter is the original file path
    (e.g., ~/.claude.json), not the backup path — derive the prefix from it.""",
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
    Read src/diet.rs focusing on:
    1. backup_claude_json() (line ~414-440) — understand backup naming pattern
    2. apply() (line ~540-573) — understand where to insert rotate_backups call
    3. format_timestamp() — confirms YYYYMMDDHHMMSS format for sorting
    """
    pass

@workflow.step(2)
def implement_rotate_backups():
    """
    Add rotate_backups() to src/diet.rs:

    1. Function signature: pub(crate) fn rotate_backups(original_path: &Path, keep: usize) -> Result<(), CyoloError>
    2. Implementation:
       a. Get parent directory from original_path
       b. Get filename stem: original_path.file_name() (e.g., "claude.json")
       c. Build prefix: "{filename}.backup-"
       d. List directory entries, filter by prefix match
       e. Collect matching paths into a Vec, sort alphabetically (ascending = oldest first)
       f. If count <= keep, return Ok(())
       g. Delete entries[0..count-keep] with fs::remove_file
       h. On individual delete failure: eprintln warning, continue

    3. Wire into apply():
       After `let backup_path = backup_claude_json(claude_json_path)?;` and its println,
       add: `rotate_backups(claude_json_path, 5)?;`
    """
    pass

@workflow.step(3)
def add_tests():
    """
    Add unit tests:
    1. test_rotate_backups_deletes_oldest: create 7 backup files with sequential timestamps,
       call rotate_backups(path, 5), verify exactly 5 remain (the newest 5)
    2. test_rotate_backups_keeps_all_when_under_limit: create 3 backups, call with keep=5,
       verify all 3 still exist
    3. test_rotate_backups_exactly_at_limit: create 5 backups, call with keep=5,
       verify all 5 still exist
    4. test_rotate_backups_no_backups: call on a dir with no backup files, verify Ok(())
    """
    pass

@workflow.step(4)
def codex_review_and_apply():
    """
    Run Codex AI code review and apply feedback:
    1. Run workflow.codex_review() to get review results
    2. Analyze the review feedback (code quality, security, performance issues)
    3. Apply necessary fixes based on the review
    4. Re-run build/test to ensure fixes don't break anything
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
    workflow.update_memory("learning", "backup-rotation", "Implemented rotate_backups with alphabetical sort of YYYYMMDDHHMMSS timestamps and tolerant delete")

@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()

if __name__ == "__main__":
    workflow.run_cli()
