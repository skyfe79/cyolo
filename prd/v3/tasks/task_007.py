#!/usr/bin/env python3
"""Task 007: Integration build verification"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-007",
    version="v3",
    title="Integration build verification",
    description="Verify that all v3 changes compile together cleanly. Run full build, test suite, and clippy. Ensure no warnings or unused code.",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="medium",
    estimated_hours=0.5,
    phase="Phase 5: Verification",
    dependencies=["task-006"],
    subtasks=[
        Subtask("task-007-1", "Run cargo build and verify clean compile", 0.1),
        Subtask("task-007-2", "Run cargo test and verify all tests pass", 0.1),
        Subtask("task-007-3", "Run cargo clippy and verify no warnings", 0.2),
    ],
    acceptance_criteria=[
        "cargo build succeeds with zero warnings",
        "cargo test passes all tests",
        "cargo clippy -- -D warnings passes with zero warnings",
        "No unused imports, dead code, or unreachable patterns",
    ],
    files=["src/main.rs", "src/cli.rs", "src/runner.rs", "src/error.rs", "src/config.rs", "src/profile.rs", "src/symlink.rs"],
    technical_notes="Final verification task. Fix any compilation issues, unused imports, or clippy warnings introduced across all v3 tasks. This is the last task before version evolution.",
    web_search=[],
)

workflow = TaskWorkflow(task)


@workflow.pre_job
def load_context():
    """Load project context."""
    workflow.retrieve_memory("learning")
    workflow.read_skills()


@workflow.step(1)
def run_full_verification():
    """
    Run the full verification suite:
    1. cargo build 2>&1 — check for compilation errors
    2. cargo test 2>&1 — check for test failures
    3. cargo clippy -- -D warnings 2>&1 — check for lint warnings

    If any step fails, investigate and fix the issue.
    Read the relevant source files to understand and resolve any errors.
    """
    pass


@workflow.step(2)
def fix_issues():
    """
    If any issues were found in step 1:
    1. Read the relevant source files
    2. Fix compilation errors, test failures, or clippy warnings
    3. Re-run the failing command to confirm the fix
    4. Repeat until all three checks pass cleanly
    """
    pass


@workflow.step(3)
def codex_review_and_apply():
    """
    Run Codex AI code review and apply feedback:
    1. Run workflow.codex_review() to get review results
    2. Analyze the review feedback (code quality, security, performance)
    3. Apply necessary fixes
    4. Re-run build/test/clippy to verify fixes
    """
    workflow.codex_review()


@workflow.verify
def check_builds():
    """Project builds without errors or warnings."""
    workflow.run_command("cargo build 2>&1")


@workflow.verify
def check_tests():
    """All tests pass."""
    workflow.run_command("cargo test 2>&1")


@workflow.verify
def check_clippy():
    """No clippy warnings."""
    workflow.run_command("cargo clippy -- -D warnings 2>&1")


@workflow.post_job
def save_learnings():
    """Record learnings."""
    workflow.update_memory("learning", "v3-integration", "v3 symlink-based config sharing: all modules compile and pass verification")


@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()
