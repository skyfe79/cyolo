#!/usr/bin/env python3
"""Task 001: Add ProfileFileError variant to error.rs"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-001",
    version="v4",
    title="Add ProfileFileError variant to error.rs",
    description="Add a ProfileFileError variant to CyoloError for invalid .claude-profile.json content. This covers cases where the file exists but has neither 'name' nor 'config_dir' fields. Malformed JSON reuses existing ConfigParseError. Unregistered name reuses ProfileNotFound.",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="high",
    estimated_hours=0.5,
    phase="Phase 1: Foundation",
    dependencies=[],
    subtasks=[
        Subtask("task-001-1", "Add ProfileFileError variant with path and message fields", 0.3),
        Subtask("task-001-2", "Verify Display output includes file path and descriptive guidance", 0.2),
    ],
    acceptance_criteria=[
        "CyoloError has ProfileFileError variant with path (PathBuf) and message (String) fields",
        "Error message includes the file path and guides the user (e.g. expected 'name' or 'config_dir' field)",
        "Existing ConfigParseError and ProfileNotFound variants are unchanged",
        "cargo build succeeds with no warnings",
        "Existing tests still pass",
    ],
    files=["src/error.rs"],
    technical_notes="Use thiserror #[error(...)] format string consistent with existing variants. The variant captures: file path of the .claude-profile.json and a descriptive message explaining what's wrong. Do NOT add variants for malformed JSON (use ConfigParseError) or unregistered profile name (use ProfileNotFound).",
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
    Note existing variants (ConfigParseError, ProfileNotFound, SymlinkError)
    and the thiserror pattern used.
    """
    pass


@workflow.step(2)
def implement_profile_file_error():
    """
    Add ProfileFileError variant to CyoloError in src/error.rs:

    ProfileFileError { path: PathBuf, message: String }
    - Display: "cyolo: invalid profile file {path}: {message}"

    Ensure PathBuf is already imported (it should be from SymlinkError).
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
    workflow.update_memory("learning", "profile-file-error-v4", "Added ProfileFileError variant for invalid .claude-profile.json content")


@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()
