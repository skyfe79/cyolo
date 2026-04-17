#!/usr/bin/env python3
"""Task 003: Update dispatch() match arms and help text for init and default"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-003",
    version="v5",
    title="Update dispatch() match arms and help text for init and default",
    description=(
        "Wire profile_init() and profile_default() into profile::dispatch() by adding "
        "match arms for 'init' and 'default' subcommands. Update the help text in the "
        "None arm to include init and default commands."
    ),
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="medium",
    estimated_hours=1,
    phase="Phase 2: Wiring",
    dependencies=["task-001", "task-002"],
    subtasks=[
        Subtask("task-003-1", "Add 'init' and 'default' match arms to dispatch()", 0.5),
        Subtask("task-003-2", "Update help text to include init and default commands", 0.5),
    ],
    acceptance_criteria=[
        "cyolo profile init work routes to profile_init()",
        "cyolo profile default routes to profile_default()",
        "cyolo profile (no subcommand) shows updated help including init and default",
        "Available commands list in the Some(cmd) error arm includes init and default",
        "cargo build succeeds",
        "cargo test passes",
    ],
    files=["src/profile.rs"],
    technical_notes=(
        "Add two new match arms in dispatch():\n"
        "  Some(\"init\") => profile_init(&args[1..]),\n"
        "  Some(\"default\") => profile_default(&args[1..]),\n"
        "Update the None arm help text to add:\n"
        "  println!(\"  init [name]              Create .claude-profile.json in current directory\");\n"
        "  println!(\"  default [name|--unset]   Get/set/clear the default profile\");\n"
        "Update the Some(cmd) error message to include init and default in the Available list."
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
def investigate_dispatch():
    """
    Read src/profile.rs dispatch() function to understand:
    - Current match arm ordering
    - Help text format in the None arm
    - Error message format in the Some(cmd) arm
    """
    pass

@workflow.step(2)
def update_dispatch():
    """
    Modify dispatch() in src/profile.rs:

    1. Add match arms BEFORE the None arm:
       Some("init") => profile_init(&args[1..]),
       Some("default") => profile_default(&args[1..]),

    2. Update the None arm help text:
       println!("Usage: cyolo profile <add|rm|list|link|current|init|default>");
       println!();
       println!("Commands:");
       println!("  add <name> [config-dir] [--no-share]  Register a new profile");
       println!("  rm <name>                Remove a profile");
       println!("  list                     List all profiles");
       println!("  link <name>              Re-create shared symlinks for a profile");
       println!("  current                  Show the currently active profile");
       println!("  init [name]              Create .claude-profile.json in current directory");
       println!("  default [name|--unset]   Get/set/clear the default profile");

    3. Update the Some(cmd) error message:
       eprintln!("Available: add, rm, list, link, current, init, default");
    """
    pass

@workflow.step(3)
def codex_review_and_apply():
    """
    Run Codex AI code review and apply feedback:
    1. Run workflow.codex_review() to get review results
    2. Analyze the review feedback (code quality, security, performance issues)
    3. Apply necessary fixes based on the review
    4. Re-run cargo build and cargo test to ensure fixes don't break anything
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
    """Record learnings from implementation."""
    workflow.update_memory("learning", "dispatch-update", "Wired init and default into profile dispatch")

@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()

if __name__ == "__main__":
    workflow.run_cli()
