#!/usr/bin/env python3
"""Task 005: Wire process check and --force into dispatch()"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-005",
    version="v7",
    title="Wire process check and --force into dispatch()",
    description="""Update dispatch() to integrate the process check gate: after parsing args,
    if !options.force, call is_claude_running(). If Claude is running, print error message
    and return Err. This is the final integration task that connects all v7 primitives
    (is_claude_running, --force, atomic_write_json, rotate_backups) into the dispatch pipeline.""",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="medium",
    estimated_hours=1,
    phase="Phase 3: Integration",
    dependencies=["task-001", "task-002", "task-003", "task-004"],
    subtasks=[
        Subtask("task-005-1", "Add process check gate in dispatch() after parse_diet_args", 0.5),
        Subtask("task-005-2", "Print error message when Claude is running without --force", 0.25),
        Subtask("task-005-3", "Verify full pipeline: cargo build + cargo test", 0.25),
    ],
    acceptance_criteria=[
        "dispatch() calls is_claude_running() after parsing args when !force",
        "Error message: 'cyolo: Claude is currently running. Stop Claude first, or use --force to proceed.'",
        "When force=true, process check is skipped",
        "Full pipeline works: parse → process check → analyze → scan → report → apply (with atomic write + rotation)",
        "cargo build succeeds",
        "cargo test passes",
    ],
    files=["src/diet.rs"],
    technical_notes="""Updated dispatch() flow:
    1. parse_diet_args(args) → DietOptions { apply, force }
    2. IF !force: is_claude_running() → if true, eprintln error + return Err(NonZeroExit(1))
    3. resolve_claude_home()
    4. analyze → scan → build report → print
    5. IF apply: backup → rotate_backups → remove_orphaned_entries (atomic) → remove_session_folders

    The error uses CyoloError::NonZeroExit(1) with eprintln for the message (same pattern as
    parse_diet_args unknown arg handling). The error message text is specified in the PRD F2.""",
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
    1. dispatch() function (line ~610-646) — understand current pipeline flow
    2. is_claude_running() — verify it exists from task-001
    3. parse_diet_args() — verify --force is handled from task-002
    4. apply() — verify rotate_backups is called from task-004
    """
    pass

@workflow.step(2)
def wire_process_check():
    """
    Modify dispatch() in src/diet.rs:

    After `let options = parse_diet_args(args)?;` add:

    ```rust
    if !options.force && is_claude_running() {
        eprintln!("cyolo: Claude is currently running. Stop Claude first, or use --force to proceed.");
        return Err(CyoloError::NonZeroExit(1));
    }
    ```

    The rest of dispatch() remains unchanged — the atomic write and backup rotation
    were already wired in by tasks 003 and 004.
    """
    pass

@workflow.step(3)
def verify_full_pipeline():
    """
    Run full verification:
    1. cargo build — ensure everything compiles together
    2. cargo test — ensure all tests pass
    3. Review the complete dispatch() function to verify correct ordering of all v7 additions
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
    workflow.update_memory("learning", "v7-integration", "Wired process check + force flag into dispatch pipeline, completing v7 safety features")

@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()

if __name__ == "__main__":
    workflow.run_cli()
