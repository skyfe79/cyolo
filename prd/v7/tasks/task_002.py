#!/usr/bin/env python3
"""Task 002: Add --force flag to DietOptions and parse_diet_args()"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-002",
    version="v7",
    title="Add --force flag to DietOptions and parse_diet_args()",
    description="""Add force: bool field to DietOptions and update parse_diet_args() to accept
    --force alongside --apply. Update usage message. Order-independent: --force --apply
    and --apply --force both work. Existing unknown-arg error behavior preserved.""",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="high",
    estimated_hours=1,
    phase="Phase 1: Safety Primitives",
    dependencies=["task-001"],
    subtasks=[
        Subtask("task-002-1", "Add force: bool to DietOptions struct", 0.25),
        Subtask("task-002-2", "Update parse_diet_args() to accept --force flag", 0.25),
        Subtask("task-002-3", "Update usage message to include --force", 0.25),
        Subtask("task-002-4", "Unit tests for --force parsing (alone, with --apply, order independence)", 0.5),
    ],
    acceptance_criteria=[
        "DietOptions has force: bool field",
        "parse_diet_args(&['--force']) returns DietOptions { apply: false, force: true }",
        "parse_diet_args(&['--apply', '--force']) returns DietOptions { apply: true, force: true }",
        "parse_diet_args(&['--force', '--apply']) returns DietOptions { apply: true, force: true }",
        "parse_diet_args(&['--unknown']) still returns error",
        "Usage message shows: cyolo diet [--apply] [--force]",
        "Existing parse_diet_args tests still pass (update test_parse_args_unknown_flag since --force is now valid)",
        "cargo test passes",
    ],
    files=["src/diet.rs"],
    technical_notes="""The existing test_parse_args_unknown_flag tests --force as unknown — this test
    must be updated since --force will now be a valid flag. Change it to test a different
    unknown flag (e.g., --verbose). Update the usage eprintln to show [--force].""",
    web_search=[],
)

workflow = TaskWorkflow(task)

@workflow.pre_job
def load_context():
    """Load project context and skill files."""
    workflow.retrieve_memory("learning")
    workflow.read_skills()

@workflow.step(1)
def investigate():
    """
    Read src/diet.rs focusing on:
    1. DietOptions struct definition (line ~10-14)
    2. parse_diet_args() function (line ~580-593)
    3. Existing test_parse_args_unknown_flag test (line ~1346-1349)
    """
    pass

@workflow.step(2)
def implement_force_flag():
    """
    Modify src/diet.rs:

    1. Add force field to DietOptions:
       ```rust
       pub(crate) struct DietOptions {
           pub apply: bool,
           pub force: bool,
       }
       ```

    2. Update parse_diet_args() to accept --force:
       ```rust
       fn parse_diet_args(args: &[String]) -> Result<DietOptions, CyoloError> {
           let mut apply = false;
           let mut force = false;
           for arg in args {
               match arg.as_str() {
                   "--apply" => apply = true,
                   "--force" => force = true,
                   _ => {
                       eprintln!("cyolo: unknown diet option '{arg}'");
                       eprintln!("Usage: cyolo diet [--apply] [--force]");
                       return Err(CyoloError::NonZeroExit(1));
                   }
               }
           }
           Ok(DietOptions { apply, force })
       }
       ```
    """
    pass

@workflow.step(3)
def update_tests():
    """
    Update existing tests and add new ones:

    1. Fix test_parse_args_unknown_flag: change --force to --verbose (since --force is now valid)
    2. Add test_parse_args_force: --force alone → { apply: false, force: true }
    3. Add test_parse_args_apply_and_force: --apply --force → { apply: true, force: true }
    4. Add test_parse_args_force_and_apply: --force --apply → { apply: true, force: true }
    5. Add test_parse_args_force_alone_no_apply: verify apply is false when only --force
    """
    pass

@workflow.step(4)
def codex_review_and_apply():
    """
    Run Codex AI code review and apply feedback:
    1. Run workflow.codex_review() to get review results
    2. Analyze the review feedback (code quality, security, performance issues)
    3. Apply necessary fixes based on the review
    4. Re-run build/test to ensure fixes don't break anything
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
    workflow.update_memory("learning", "force-flag", "Added --force flag to DietOptions and parse_diet_args with order-independent parsing")

@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()

if __name__ == "__main__":
    workflow.run_cli()
