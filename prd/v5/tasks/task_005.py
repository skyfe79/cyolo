#!/usr/bin/env python3
"""Task 005: v5 integration build verification"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-005",
    version="v5",
    title="v5 integration build verification",
    description=(
        "Final verification that v5 builds cleanly, all tests pass, and clippy reports "
        "no warnings. Confirms the complete profile init and default management feature set."
    ),
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="low",
    estimated_hours=1,
    phase="Phase 4: Verification",
    dependencies=["task-004"],
    subtasks=[
        Subtask("task-005-1", "Run cargo build and verify success", 0.25),
        Subtask("task-005-2", "Run cargo test and verify all pass", 0.25),
        Subtask("task-005-3", "Run cargo clippy and fix any warnings", 0.5),
    ],
    acceptance_criteria=[
        "cargo build completes with exit code 0",
        "cargo test completes with all tests passing",
        "cargo clippy -- -D warnings reports no warnings",
        "No dead_code warnings from new functions",
    ],
    files=["src/profile.rs", "src/config.rs"],
    technical_notes=(
        "This is a verification-only task. If clippy reports warnings on the new code, "
        "fix them in this task. Common issues: unused variables, missing docs on pub functions, "
        "redundant clones. If all checks pass on first run, mark as completed quickly."
    ),
    web_search=[],
)

workflow = TaskWorkflow(task)

@workflow.pre_job
def load_context():
    """Load project context."""
    workflow.retrieve_memory("learning")

@workflow.step(1)
def run_full_build():
    """
    Run the full build and test suite:
    1. cargo build 2>&1 — check for compilation errors
    2. cargo test 2>&1 — check all tests pass
    3. cargo clippy -- -D warnings 2>&1 — check for lint warnings
    If any step fails, diagnose and fix the issue.
    """
    pass

@workflow.step(2)
def codex_review_and_apply():
    """
    Run Codex AI code review and apply feedback:
    1. Run workflow.codex_review() to review the complete v5 changeset
    2. Analyze review feedback focusing on overall code quality
    3. Apply any final fixes
    4. Re-run cargo build + test + clippy to verify
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

@workflow.verify
def check_clippy():
    """No clippy warnings."""
    workflow.run_command("cargo clippy -- -D warnings")

@workflow.post_job
def save_learnings():
    """Record v5 completion."""
    workflow.update_memory("learning", "v5-complete", "v5 profile init and default management complete")

@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()

if __name__ == "__main__":
    workflow.run_cli()
