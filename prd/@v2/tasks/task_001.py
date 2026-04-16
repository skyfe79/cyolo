#!/usr/bin/env python3
"""Task 001: Extend error types for config and profile operations"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-001",
    version="v2",
    title="Extend error types for config and profile operations",
    description="Add ConfigParseError, ConfigIoError, ProfileAlreadyExists, and ProfileNotFound variants to CyoloError in error.rs. Each variant should produce a user-friendly error message. ConfigParseError includes the file path. ProfileNotFound suggests 'cyolo profile add <name>'.",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="high",
    estimated_hours=1,
    phase="Phase 1: Foundation",
    dependencies=[],
    subtasks=[
        Subtask("task-001-1", "Add ConfigParseError variant with file path field", 0.3),
        Subtask("task-001-2", "Add ConfigIoError variant wrapping std::io::Error", 0.2),
        Subtask("task-001-3", "Add ProfileAlreadyExists variant with name field", 0.2),
        Subtask("task-001-4", "Add ProfileNotFound variant with suggestion message", 0.3),
    ],
    acceptance_criteria=[
        "CyoloError has ConfigParseError variant with path field",
        "CyoloError has ConfigIoError variant wrapping io::Error",
        "CyoloError has ProfileAlreadyExists variant showing profile name",
        "CyoloError has ProfileNotFound variant suggesting 'cyolo profile add <name>'",
        "cargo build succeeds with no warnings",
        "Existing tests still pass",
    ],
    files=["src/error.rs"],
    technical_notes="Use thiserror #[error(...)] format strings. ConfigParseError needs both a path (PathBuf) and source (serde_json::Error). ConfigIoError wraps io::Error with a context message. ProfileNotFound message should say: profile '<name>' not found. Run: cyolo profile add <name>",
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
    Note the existing variants and thiserror pattern used.
    """
    pass


@workflow.step(2)
def implement_error_variants():
    """
    Add four new error variants to CyoloError in src/error.rs:

    1. ConfigParseError { path: PathBuf, source: serde_json::Error }
       - Message: "cyolo: failed to parse config at {path}: {source}"

    2. ConfigIoError { context: String, source: std::io::Error }
       - Message: "cyolo: {context}: {source}"

    3. ProfileAlreadyExists { name: String }
       - Message: "cyolo: profile '{name}' already exists"

    4. ProfileNotFound { name: String }
       - Message: "cyolo: profile '{name}' not found. Run: cyolo profile add {name}"

    Add necessary use statements (serde_json is already in Cargo.toml).
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
    workflow.update_memory("learning", "error-types-v2", "Extended CyoloError with config/profile variants using thiserror")


@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()
