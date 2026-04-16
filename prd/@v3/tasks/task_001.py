#!/usr/bin/env python3
"""Task 001: Add SymlinkError variant to error.rs"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-001",
    version="v3",
    title="Add SymlinkError variant to error.rs",
    description="Add a SymlinkError variant to CyoloError for symlink creation failures. Most symlink edge cases (missing source, existing target) are warnings printed to stderr, not hard errors. SymlinkError covers unexpected I/O failures only.",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="high",
    estimated_hours=0.5,
    phase="Phase 1: Foundation",
    dependencies=[],
    subtasks=[
        Subtask("task-001-1", "Add SymlinkError variant with item, source, target fields", 0.3),
        Subtask("task-001-2", "Implement Display for SymlinkError with descriptive message", 0.2),
    ],
    acceptance_criteria=[
        "CyoloError has SymlinkError variant with item (String), source (PathBuf), target (PathBuf), and source_err (std::io::Error) fields",
        "Error message includes item name, source path, and target path",
        "cargo build succeeds with no warnings",
        "Existing tests still pass",
    ],
    files=["src/error.rs"],
    technical_notes="Use thiserror #[error(...)] format string. The variant captures: item name (e.g. 'CLAUDE.md'), source path, target path, and the underlying io::Error. Warning messages for skip/existing-target are handled in symlink.rs via eprintln!, not as CyoloError variants.",
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
    Read src/error.rs to understand the current CyoloError enum structure.
    Note existing variants and the thiserror pattern used in v2.
    """
    pass


@workflow.step(2)
def implement_symlink_error():
    """
    Add SymlinkError variant to CyoloError in src/error.rs:

    SymlinkError { item: String, source: PathBuf, target: PathBuf, source_err: std::io::Error }
    - Message: "cyolo: failed to symlink {item}: {source} -> {target}: {source_err}"

    Add necessary use statements (PathBuf from std::path).
    """
    pass


@workflow.step(3)
def codex_review_and_apply():
    """
    Run Codex AI code review and apply feedback:
    1. Run workflow.codex_review() to get review results
    2. Analyze the review feedback
    3. Apply necessary fixes
    4. Re-run cargo build to verify fixes
    """
    workflow.codex_review()


@workflow.verify
def check_builds():
    """Project builds without errors."""
    workflow.run_command("cargo build 2>&1")


@workflow.verify
def check_tests():
    """Existing tests pass."""
    workflow.run_command("cargo test 2>&1")


@workflow.post_job
def save_learnings():
    """Record learnings."""
    workflow.update_memory("learning", "symlink-error-v3", "Added SymlinkError variant to CyoloError for symlink I/O failures")


@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()
