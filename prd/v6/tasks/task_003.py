#!/usr/bin/env python3
"""Task 003: Implement scan_session_folders() — orphaned session directory detection"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-003",
    version="v6",
    title="Implement scan_session_folders() for orphaned session directory detection",
    description="""Match orphaned project paths to their session directories in ~/.claude/projects/.
    Implement path encoding (/ -> -), calculate total size of each orphaned session folder,
    and handle missing ~/.claude/projects/ directory gracefully.""",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="high",
    estimated_hours=3,
    phase="Phase 2: Analysis",
    dependencies=["task-001"],
    subtasks=[
        Subtask("task-003-1", "Implement project_path_to_session_dir() path encoding helper", 0.5),
        Subtask("task-003-2", "Implement dir_size() recursive size calculation", 1),
        Subtask("task-003-3", "Implement scan_session_folders() to find orphaned session dirs", 1),
        Subtask("task-003-4", "Unit tests for path encoding and session folder scanning", 1),
    ],
    acceptance_criteria=[
        "Correctly maps project paths to session folder names using '-' encoding",
        "project_path_to_session_dir('/Users/codingmax/Private/labs/test-bot') returns '-Users-codingmax-Private-labs-test-bot'",
        "Accurately calculates total size of session folder contents",
        "Handles missing ~/.claude/projects/ directory gracefully (returns empty results)",
        "Also calculates total session_dir_size for the report",
        "cargo test passes",
    ],
    files=["src/diet.rs"],
    technical_notes="""Path encoding: replace '/' with '-' in the absolute project path.
    This is Claude Code's own convention.

    dir_size() should use fs::read_dir with manual recursion (session dirs are shallow).
    Use fs::metadata().len() for file sizes. Skip entries that fail to read (permissions).

    scan_session_folders takes:
    - projects_dir: &Path (e.g., ~/.claude/projects/)
    - orphaned_paths: &[String] (from analyze step)
    Returns (Vec<OrphanedSession>, u64 total_session_dir_size).""",
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
    Read src/diet.rs to understand current data structures.
    Understand the OrphanedSession struct fields.
    """
    pass

@workflow.step(2)
def implement_helpers():
    """
    1. Implement project_path_to_session_dir(project_path: &str) -> String:
       - project_path.replace('/', "-")

    2. Implement dir_size(path: &Path) -> u64:
       - If path doesn't exist or isn't a directory, return 0
       - Read directory entries recursively
       - Sum fs::metadata().len() for each file
       - Skip entries that fail (permission errors)
       - For subdirectories, recurse
    """
    pass

@workflow.step(3)
def implement_scan():
    """
    Implement scan_session_folders(projects_dir: &Path, orphaned_paths: &[String]) -> (Vec<OrphanedSession>, u64):

    1. If projects_dir doesn't exist, return (empty vec, 0)
    2. Calculate total_session_dir_size by scanning all entries in projects_dir
    3. For each orphaned_path:
       - Convert to session folder name via project_path_to_session_dir()
       - Check if projects_dir.join(session_name) exists
       - If exists: calculate dir_size(), create OrphanedSession
    4. Return (orphaned_sessions, total_session_dir_size)
    """
    pass

@workflow.step(4)
def add_tests():
    """
    Add unit tests:
    - test_path_encoding: verify '/' -> '-' conversion for various paths
    - test_path_encoding_root: "/" -> "-"
    - test_dir_size_empty: empty dir -> 0
    - test_dir_size_with_files: dir with known-size files -> correct sum
    - test_scan_missing_projects_dir: projects_dir doesn't exist -> empty
    - test_scan_finds_orphaned_sessions: create mock session dirs matching orphaned paths
    """
    pass

@workflow.step(5)
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
    workflow.update_memory("learning", "diet-scan-sessions", "Implemented session folder scanning with path encoding and recursive size calculation")

@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()

if __name__ == "__main__":
    workflow.run_cli()
