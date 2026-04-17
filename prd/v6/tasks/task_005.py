#!/usr/bin/env python3
"""Task 005: Implement apply() — backup and remove orphaned entries"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-005",
    version="v6",
    title="Implement apply() for backup and orphaned entry removal",
    description="""Execute the cleanup when --apply is provided. Create backup of ~/.claude.json,
    remove orphaned keys from the parsed serde_json::Value, write back to file,
    and remove orphaned session folders with symlink safety.""",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="medium",
    estimated_hours=4,
    phase="Phase 4: Apply",
    dependencies=["task-002", "task-003"],
    subtasks=[
        Subtask("task-005-1", "Implement backup_claude_json() with YYYYMMDDHHMMSS timestamp", 1),
        Subtask("task-005-2", "Implement remove_orphaned_entries() on serde_json::Value + fs::write", 1),
        Subtask("task-005-3", "Implement remove_session_folders() with symlink safety", 1),
        Subtask("task-005-4", "Implement apply() orchestrating backup + remove + folder cleanup", 0.5),
        Subtask("task-005-5", "Unit tests for backup, remove, and symlink safety", 1),
    ],
    acceptance_criteria=[
        "Backup file created as ~/.claude.json.backup-<YYYYMMDDHHMMSS> before any modification",
        "~/.claude.json on disk has orphaned keys removed, all other fields preserved",
        "Orphaned session folders deleted from ~/.claude/projects/",
        "Symlinks in session folders are unlinked (not followed) using fs::remove_file",
        "Regular files/dirs removed with fs::remove_dir_all",
        "Prints summary: backup path, N entries removed, N folders removed, size freed",
        "cargo test passes",
    ],
    files=["src/diet.rs"],
    technical_notes="""Backup timestamp: use SystemTime::now().duration_since(UNIX_EPOCH) then
    manual arithmetic for YYYYMMDDHHMMSS (no chrono dependency).

    Remove orphaned entries: projects.as_object_mut().retain(|path, _| !orphaned_set.contains(path))
    Write back: serde_json::to_string_pretty(&value) + fs::write (simple write, not atomic — v7 adds atomic).

    Symlink safety for session folder removal:
    - For each entry in session folder, check fs::symlink_metadata()
    - If metadata.file_type().is_symlink(): fs::remove_file() (unlink only)
    - Else: proceed with normal removal
    - After clearing symlinks, fs::remove_dir_all() for the folder

    Timestamp helper: format_timestamp(secs: u64) -> String for YYYYMMDDHHMMSS.
    Calculate year/month/day/hour/min/sec from unix timestamp manually.""",
    web_search=[],
)

workflow = TaskWorkflow(task)

@workflow.pre_job
def load_context():
    """Load project context."""
    workflow.retrieve_memory("learning")
    workflow.read_skills()

@workflow.step(1)
def investigate():
    """
    Read src/diet.rs for current state after task-001/002/003.
    Read src/config.rs for atomic_write pattern (reference for v7, v6 uses simple write).
    """
    pass

@workflow.step(2)
def implement_backup():
    """
    Implement backup_claude_json(claude_json_path: &Path) -> Result<PathBuf, CyoloError>:

    1. Get current time: SystemTime::now().duration_since(UNIX_EPOCH)
    2. Format timestamp as YYYYMMDDHHMMSS using manual arithmetic:
       - secs since epoch -> days -> year/month/day
       - remainder -> hours/minutes/seconds
       - Use a helper: format_timestamp(secs_since_epoch: u64) -> String
    3. Construct backup path: claude_json_path with ".backup-{timestamp}" appended
    4. fs::copy(claude_json_path, &backup_path)
    5. Return Ok(backup_path)
    """
    pass

@workflow.step(3)
def implement_remove_entries():
    """
    Implement remove_orphaned_entries(
        parsed_json: &mut serde_json::Value,
        orphaned_paths: &[String],
        claude_json_path: &Path,
    ) -> Result<(), CyoloError>:

    1. Get projects as mutable object: parsed_json.get_mut("projects").as_object_mut()
    2. Create HashSet from orphaned_paths for O(1) lookup
    3. projects.retain(|path, _| !orphaned_set.contains(path.as_str()))
    4. Serialize: serde_json::to_string_pretty(&parsed_json)
    5. Write: fs::write(claude_json_path, &serialized)
    6. Return Ok(())
    """
    pass

@workflow.step(4)
def implement_remove_folders():
    """
    Implement remove_session_folders(sessions: &[OrphanedSession]) -> Result<(usize, u64), CyoloError>:

    1. For each session folder:
       a. Read directory entries
       b. For each entry: check fs::symlink_metadata()
       c. If is_symlink(): fs::remove_file() (unlink, don't follow)
       d. After processing symlinks, fs::remove_dir_all() for remaining content
    2. Track count of removed folders and total bytes freed
    3. Return Ok((removed_count, bytes_freed))
    """
    pass

@workflow.step(5)
def implement_apply():
    """
    Implement apply(
        report: &DietReport,
        parsed_json: &mut serde_json::Value,
        claude_json_path: &Path,
    ) -> Result<(), CyoloError>:

    1. Call backup_claude_json(claude_json_path) -> print backup path
    2. Collect orphaned paths from report.orphaned_projects
    3. Call remove_orphaned_entries(parsed_json, &orphaned_paths, claude_json_path)
    4. Print "Removed N orphaned project entries"
    5. Call remove_session_folders(&report.orphaned_sessions)
    6. Print "Removed N orphaned session folders (N.N MB freed)"
    """
    pass

@workflow.step(6)
def add_tests():
    """
    Add unit tests:
    - test_format_timestamp: known unix timestamp -> correct YYYYMMDDHHMMSS
    - test_backup_creates_file: verify backup file exists after call
    - test_remove_entries_preserves_other_fields: orphans removed, other keys intact
    - test_remove_session_folders_with_symlinks: symlinks unlinked, not followed
    - test_remove_session_folders_regular: regular dirs removed completely
    """
    pass

@workflow.step(7)
def codex_review_and_apply():
    """
    Run Codex AI code review and apply feedback:
    1. Run workflow.codex_review() to get review results
    2. Analyze the review feedback
    3. Apply necessary fixes
    4. Re-run cargo build/test to verify
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
    workflow.update_memory("learning", "diet-apply", "Implemented apply with backup, orphan removal, and symlink-safe folder deletion")

@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()

if __name__ == "__main__":
    workflow.run_cli()
