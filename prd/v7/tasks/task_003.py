#!/usr/bin/env python3
"""Task 003: Implement atomic_write_json() and replace fs::write in remove_orphaned_entries()"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-003",
    version="v7",
    title="Implement atomic_write_json() and replace fs::write in remove_orphaned_entries()",
    description="""Create atomic_write_json() that writes JSON content to a temp file in the same
    directory, calls sync_all(), then atomically renames to the target path. Replace the
    fs::write call in remove_orphaned_entries() with atomic_write_json(). Pattern matches
    config.rs::save() which already uses tempfile + fsync + rename.""",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="high",
    estimated_hours=1.5,
    phase="Phase 2: Atomic Operations",
    dependencies=[],
    subtasks=[
        Subtask("task-003-1", "Implement atomic_write_json(path, content) -> Result<(), CyoloError>", 0.5),
        Subtask("task-003-2", "Replace fs::write in remove_orphaned_entries() with atomic_write_json()", 0.25),
        Subtask("task-003-3", "Unit tests for atomic_write_json (basic write, sync+rename verified)", 0.75),
    ],
    acceptance_criteria=[
        "atomic_write_json() exists in src/diet.rs",
        "Temp file uses path.with_extension('json.tmp') in same directory as target",
        "sync_all() is called before rename()",
        "remove_orphaned_entries() uses atomic_write_json() instead of fs::write",
        "Existing tests for remove_orphaned_entries still pass",
        "cargo test passes",
        "cargo build succeeds",
    ],
    files=["src/diet.rs", "src/config.rs"],
    technical_notes="""Add `use std::io::Write;` import for write_all() + sync_all() methods.
    The existing `use std::fmt::Write as _;` is for string formatting — keep both.
    Reference src/config.rs::save() for the same pattern already used in this codebase.
    Temp file path: claude_json_path.with_extension("json.tmp").
    Error handling: wrap each step (create, write_all, sync_all, rename) with CyoloError::ConfigIoError.""",
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
    Read existing source files:
    1. Read src/config.rs — find the atomic write pattern (save() method) to replicate
    2. Read src/diet.rs — focus on remove_orphaned_entries() (line ~447-473) and imports
    """
    pass

@workflow.step(2)
def implement_atomic_write():
    """
    Add atomic_write_json() to src/diet.rs:

    1. Add `use std::io::Write;` import alongside existing imports
    2. Implement the function:
       ```rust
       fn atomic_write_json(path: &Path, content: &str) -> Result<(), CyoloError> {
           let tmp_path = path.with_extension("json.tmp");
           let mut file = fs::File::create(&tmp_path).map_err(|e| CyoloError::ConfigIoError {
               context: format!("failed to create temp file {}", tmp_path.display()),
               source: e,
           })?;
           file.write_all(content.as_bytes()).map_err(|e| CyoloError::ConfigIoError {
               context: format!("failed to write temp file {}", tmp_path.display()),
               source: e,
           })?;
           file.sync_all().map_err(|e| CyoloError::ConfigIoError {
               context: format!("failed to sync temp file {}", tmp_path.display()),
               source: e,
           })?;
           fs::rename(&tmp_path, path).map_err(|e| CyoloError::ConfigIoError {
               context: format!("failed to rename {} to {}", tmp_path.display(), path.display()),
               source: e,
           })?;
           Ok(())
       }
       ```

    3. Replace fs::write in remove_orphaned_entries():
       Change `fs::write(claude_json_path, serialized)` to `atomic_write_json(claude_json_path, &serialized)`
    """
    pass

@workflow.step(3)
def add_tests():
    """
    Add unit tests:
    1. test_atomic_write_json_basic: write content to a temp dir path, verify file contents match
    2. test_atomic_write_json_no_leftover_tmp: verify .json.tmp is gone after successful write
    3. Existing remove_orphaned_entries tests should still pass (they verify final file content)
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
    workflow.update_memory("learning", "atomic-write", "Implemented atomic_write_json with tempfile + sync_all + rename pattern matching config.rs")

@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()

if __name__ == "__main__":
    workflow.run_cli()
