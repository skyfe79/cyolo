#!/usr/bin/env python3
"""Task 007: Wire diet into CLI and full build verification"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-007",
    version="v6",
    title="Wire diet::dispatch() into cli.rs and verify full build",
    description="""Connect diet::dispatch() to the CLI routing in cli.rs. Replace the
    NotImplemented stub with the actual diet::dispatch() call. Verify the entire project
    builds and all tests pass.""",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="medium",
    estimated_hours=1,
    phase="Phase 5: Integration",
    dependencies=["task-006"],
    subtasks=[
        Subtask("task-007-1", "Update cli.rs: replace NotImplemented with diet::dispatch(&args)", 0.5),
        Subtask("task-007-2", "Add 'use crate::diet;' import to cli.rs", 0.25),
        Subtask("task-007-3", "Full cargo build + cargo test verification", 0.5),
    ],
    acceptance_criteria=[
        "cli.rs routes Command::Diet(args) to diet::dispatch(&args)",
        "'use crate::diet;' import added to cli.rs",
        "cargo build succeeds with no warnings",
        "cargo test passes (all existing + new tests)",
        "cargo clippy passes (if available)",
    ],
    files=["src/cli.rs"],
    technical_notes="""Change in cli.rs route() function:
    FROM: Command::Diet(_) => Err(CyoloError::NotImplemented("diet".into()))
    TO:   Command::Diet(args) => diet::dispatch(&args)

    Also add import at top: use crate::diet;

    Note: mod diet is already added in main.rs by task-001.
    This is a small task but critical — it's the integration point.""",
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
    Read src/cli.rs to see current Diet routing.
    Read src/main.rs to confirm mod diet is declared.
    Quick cargo build to ensure current state compiles.
    """
    pass

@workflow.step(2)
def wire_diet():
    """
    1. Add 'use crate::diet;' to the imports in cli.rs
    2. Change the Diet arm in route():
       FROM: Command::Diet(_) => Err(CyoloError::NotImplemented("diet".into()))
       TO:   Command::Diet(args) => diet::dispatch(&args)
    """
    pass

@workflow.step(3)
def full_verification():
    """
    Run full verification:
    1. cargo build -- verify no compilation errors
    2. cargo test -- verify all tests pass
    3. cargo clippy (if available) -- verify no lint warnings
    """
    pass

@workflow.step(4)
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
    workflow.update_memory("learning", "diet-wiring", "Wired diet::dispatch into cli.rs, replacing NotImplemented stub")

@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()

if __name__ == "__main__":
    workflow.run_cli()
