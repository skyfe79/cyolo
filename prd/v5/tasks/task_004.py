#!/usr/bin/env python3
"""Task 004: Unit tests for profile_default and profile_init"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-004",
    version="v5",
    title="Unit tests for profile_default and profile_init edge cases",
    description=(
        "Add unit tests in src/profile.rs #[cfg(test)] module covering the key edge cases "
        "for profile_default() and profile_init(). Focus on argument parsing, error conditions, "
        "and dispatch routing."
    ),
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="medium",
    estimated_hours=2,
    phase="Phase 3: Testing",
    dependencies=["task-003"],
    subtasks=[
        Subtask("task-004-1", "Add tests for profile_default argument parsing and errors", 1),
        Subtask("task-004-2", "Add tests for profile_init argument parsing and errors", 0.5),
        Subtask("task-004-3", "Add tests for dispatch routing of init and default", 0.5),
    ],
    acceptance_criteria=[
        "Test: dispatch routes 'init' to profile_init",
        "Test: dispatch routes 'default' to profile_default",
        "Test: profile_default with too many args returns error",
        "Test: profile_init with too many args returns error",
        "All new tests pass with cargo test",
        "No regressions in existing tests",
    ],
    files=["src/profile.rs"],
    technical_notes=(
        "Existing test pattern in profile.rs: helper fn args(strs: &[&str]) -> Vec<String>. "
        "Tests that touch the filesystem (config load/save) require tempdir setup. "
        "For argument-parsing-only tests, we can test the functions directly. "
        "Note: profile_default and profile_init call CyoloConfig::load() internally, "
        "so integration-style tests that need a real config will be limited. "
        "Focus on testing behaviors that can be verified without filesystem mocking: "
        "dispatch routing, argument count validation, usage error messages."
    ),
    web_search=[],
)

workflow = TaskWorkflow(task)

@workflow.pre_job
def load_context():
    """Load project context and skill files."""
    workflow.retrieve_memory("learning")
    workflow.read_skills()

@workflow.step(1)
def investigate_test_patterns():
    """
    Read existing tests in src/profile.rs to understand:
    - Test module structure (#[cfg(test)] mod tests)
    - Helper functions (args())
    - How dispatch errors are tested
    - What test infrastructure is available
    """
    pass

@workflow.step(2)
def implement_tests():
    """
    Add unit tests to the #[cfg(test)] mod tests in src/profile.rs:

    1. Dispatch routing tests:
       - test_dispatch_init_routes_correctly: dispatch(&args(&["init"])) should not
         return unknown-command error (may still fail due to missing config, but
         should not hit the "unknown profile command" path)
       - test_dispatch_default_routes_correctly: same for "default"

    2. Argument validation tests (these test the functions directly):
       - test_profile_default_too_many_args: profile_default(&args(&["a", "b"])) → error
       - test_profile_init_too_many_args: profile_init(&args(&["a", "b"])) → error
         (if profile_init rejects >1 positional arg)

    3. Help text verification:
       - test_dispatch_no_args_shows_help: dispatch(&[]) → Ok(()) (prints help)
       - Verify the help output conceptually includes 'init' and 'default'
         (this is already tested by dispatch routing)

    Note: Tests that need a valid CyoloConfig (like set/get default with a real profile)
    require filesystem setup. If feasible, add one integration test using a tempdir.
    If not, document that these are covered by the integration build in task-005.
    """
    pass

@workflow.step(3)
def codex_review_and_apply():
    """
    Run Codex AI code review and apply feedback:
    1. Run workflow.codex_review() to get review results
    2. Analyze the review feedback (code quality, security, performance issues)
    3. Apply necessary fixes based on the review
    4. Re-run cargo test to ensure fixes don't break anything
    """
    workflow.codex_review()

@workflow.verify
def check_builds():
    """Project builds without errors."""
    workflow.run_command("cargo build")

@workflow.verify
def check_tests_pass():
    """All tests pass including new tests."""
    workflow.run_command("cargo test")

@workflow.post_job
def save_learnings():
    """Record learnings from testing."""
    workflow.update_memory("learning", "v5-tests", "Added unit tests for profile default and init")

@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()

if __name__ == "__main__":
    workflow.run_cli()
