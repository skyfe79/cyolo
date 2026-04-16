#!/usr/bin/env python3
"""Task 007: Integration build verification"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-007",
    version="v2",
    title="Integration build verification",
    description="Final verification that all v2 features compile, tests pass, and the binary runs correctly. Verify cargo build, cargo test, and basic CLI invocations for profile add/rm/list. Check no compiler warnings.",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="medium",
    estimated_hours=1,
    phase="Phase 3: Integration",
    dependencies=["task-006"],
    subtasks=[
        Subtask("task-007-1", "Run cargo build and verify zero warnings", 0.3),
        Subtask("task-007-2", "Run cargo test and verify all pass", 0.3),
        Subtask("task-007-3", "Run cargo clippy for lint checks", 0.4),
    ],
    acceptance_criteria=[
        "cargo build succeeds with no warnings",
        "cargo test passes all tests",
        "cargo clippy reports no warnings",
        "Binary is produced at target/debug/cyolo",
        "Module structure matches PRD: main.rs, cli.rs, runner.rs, error.rs, config.rs, profile.rs",
    ],
    files=["src/main.rs", "src/cli.rs", "src/runner.rs", "src/error.rs", "src/config.rs", "src/profile.rs", "Cargo.toml"],
    technical_notes="This is a verification-only task. Fix any warnings or test failures found. Run clippy with: cargo clippy -- -D warnings. Verify the module structure in src/ matches the expected 6 files.",
    web_search=[],
)

workflow = TaskWorkflow(task)


@workflow.pre_job
def load_context():
    """Load project context."""
    workflow.retrieve_memory("learning")
    workflow.read_skills()


@workflow.step(1)
def run_build():
    """
    Run cargo build and check for any warnings or errors.
    If warnings exist, fix them.
    """
    pass


@workflow.step(2)
def run_tests():
    """
    Run cargo test and verify all tests pass.
    If any tests fail, investigate and fix.
    """
    pass


@workflow.step(3)
def run_clippy():
    """
    Run cargo clippy -- -D warnings.
    Fix any clippy warnings found.
    """
    pass


@workflow.step(4)
def verify_module_structure():
    """
    Verify src/ contains exactly:
    - main.rs, cli.rs, runner.rs, error.rs, config.rs, profile.rs
    Verify main.rs has: mod cli, mod error, mod runner, mod config, mod profile
    """
    pass


@workflow.step(5)
def codex_review_and_apply():
    """
    Run Codex AI code review and apply feedback:
    1. Run workflow.codex_review() for final code quality check
    2. Analyze the review feedback
    3. Apply any remaining fixes
    4. Re-run cargo build + test + clippy to verify
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
    workflow.update_memory("learning", "v2-integration", "v2 integration verified: config + profile modules complete")


@workflow.post_job
def commit_changes():
    """Commit any fixes."""
    workflow.require_review_log()
    workflow.git_commit()
