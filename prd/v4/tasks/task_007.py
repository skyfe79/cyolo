#!/usr/bin/env python3
"""Task 007: Integration build verification for all v4 features"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-007",
    version="v4",
    title="Integration build verification for all v4 features",
    description="Verify the complete v4 feature set builds and works together. Run cargo build, cargo test, cargo clippy. Check that detect.rs, runner.rs, cli.rs, profile.rs, and error.rs are all properly wired. Fix any warnings or integration issues.",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="medium",
    estimated_hours=1,
    phase="Phase 4: Verification",
    dependencies=["task-005", "task-006"],
    subtasks=[
        Subtask("task-007-1", "Run cargo build and fix any errors", 0.3),
        Subtask("task-007-2", "Run cargo test and fix any failures", 0.3),
        Subtask("task-007-3", "Run cargo clippy and fix any warnings", 0.2),
        Subtask("task-007-4", "Verify all modules are properly connected", 0.2),
    ],
    acceptance_criteria=[
        "cargo build succeeds with zero errors and zero warnings",
        "cargo test succeeds with all tests passing",
        "cargo clippy produces no warnings",
        "detect.rs is properly declared as mod and accessible from cli.rs and profile.rs",
        "resolve_profile() is called in Command::Claude path",
        "profile current subcommand compiles and is reachable",
        "CLAUDE_CONFIG_DIR env var is set on child Command when profile detected",
    ],
    files=["src/detect.rs", "src/runner.rs", "src/cli.rs", "src/profile.rs", "src/error.rs", "src/main.rs"],
    technical_notes="This is a verification task. Read all modified source files and confirm the integration is correct. Fix any compiler warnings, clippy lints, or test failures. Do NOT add new features — only fix integration issues found during verification.",
    web_search=[],
)

workflow = TaskWorkflow(task)


@workflow.pre_job
def load_context():
    """Load project context."""
    workflow.retrieve_memory("learning")
    workflow.read_skills()


@workflow.step(1)
def run_full_build():
    """
    Run cargo build 2>&1 and check for any errors or warnings.
    If any issues found, read the relevant source files and fix them.
    """
    pass


@workflow.step(2)
def run_tests():
    """
    Run cargo test 2>&1 and check for any test failures.
    Fix failing tests if they indicate real integration issues.
    """
    pass


@workflow.step(3)
def run_clippy():
    """
    Run cargo clippy 2>&1 and check for any warnings.
    Fix clippy warnings to ensure clean code quality.
    """
    pass


@workflow.step(4)
def verify_integration():
    """
    Read all modified source files and verify:
    1. main.rs declares mod detect
    2. cli.rs calls detect::resolve_profile() in Command::Claude
    3. runner.rs run_claude() accepts and uses config_dir
    4. profile.rs dispatch handles "current" subcommand
    5. error.rs has ProfileFileError variant
    """
    pass


@workflow.step(5)
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
    """All tests pass."""
    workflow.run_command("cargo test 2>&1")


@workflow.verify
def check_clippy():
    """No clippy warnings."""
    workflow.run_command("cargo clippy 2>&1")


@workflow.post_job
def save_learnings():
    """Record learnings."""
    workflow.update_memory("learning", "integration-v4", "v4 integration verification passed: profile detection + walk-up fully wired")


@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()
