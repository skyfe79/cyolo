#!/usr/bin/env python3
"""Task 006: Implement profile dispatch and update CLI routing"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-006",
    version="v2",
    title="Implement profile dispatch and update CLI routing",
    description="Create the profile::dispatch() function that routes profile subcommands (add, rm, list) and handles missing/unknown subcommands. Update cli.rs to replace the NotImplemented('profile') stub with a call to profile::dispatch().",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="high",
    estimated_hours=2,
    phase="Phase 3: Integration",
    dependencies=["task-003", "task-004", "task-005"],
    subtasks=[
        Subtask("task-006-1", "Create dispatch() function with match routing in profile.rs", 0.5),
        Subtask("task-006-2", "Handle no-args case with usage hint", 0.3),
        Subtask("task-006-3", "Handle unknown subcommand with available list", 0.3),
        Subtask("task-006-4", "Update cli.rs to call profile::dispatch()", 0.5),
        Subtask("task-006-5", "Add mod config and mod profile to main.rs if not already present", 0.4),
    ],
    acceptance_criteria=[
        "'cyolo profile add work' dispatches to profile::add",
        "'cyolo profile list' dispatches to profile::list",
        "'cyolo profile rm work' dispatches to profile::rm",
        "'cyolo profile' with no args prints usage hint",
        "'cyolo profile unknown' prints error with available subcommands",
        "Other commands (update, diet, pass-through) unchanged",
        "cli.rs no longer returns NotImplemented for profile",
        "cargo build succeeds",
        "All existing tests pass",
    ],
    files=["src/profile.rs", "src/cli.rs", "src/main.rs"],
    technical_notes="pub fn dispatch(args: &[String]) -> Result<(), CyoloError>. Match on args.first().map(|s| s.as_str()): Some('add') => add(&args[1..]), Some('rm')|Some('remove') => rm(&args[1..]), Some('list')|Some('ls') => list(), None => print usage, Some(unknown) => print error with available subcommands. In cli.rs, change Command::Profile(args) => profile::dispatch(&args). Add `use crate::profile;` if needed.",
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
    Read src/cli.rs to see the current Command::Profile routing (NotImplemented stub).
    Read src/profile.rs to see all implemented functions (add, rm, list).
    Read src/main.rs for module declarations.
    """
    pass


@workflow.step(2)
def implement_dispatch():
    """
    Add to src/profile.rs:

    pub fn dispatch(args: &[String]) -> Result<(), CyoloError> {
        match args.first().map(|s| s.as_str()) {
            Some("add") => add(&args[1..]),
            Some("rm") | Some("remove") => rm(&args[1..]),
            Some("list") | Some("ls") => list(),
            None => {
                println!("Usage: cyolo profile <add|rm|list>");
                println!();
                println!("Commands:");
                println!("  add <name> [config-dir]  Register a new profile");
                println!("  rm <name>                Remove a profile");
                println!("  list                     List all profiles");
                Ok(())
            }
            Some(cmd) => {
                eprintln!("cyolo: unknown profile command '{cmd}'");
                eprintln!("Available: add, rm, list");
                // Return a suitable error or just Ok(()) after printing
                Ok(())
            }
        }
    }
    """
    pass


@workflow.step(3)
def update_cli_routing():
    """
    In src/cli.rs:
    1. Add `use crate::profile;` import
    2. Change `Command::Profile(_) => Err(CyoloError::NotImplemented("profile".into()))`
       to `Command::Profile(args) => profile::dispatch(&args)`

    In src/main.rs:
    - Ensure `mod config;` and `mod profile;` are declared
    """
    pass


@workflow.step(4)
def codex_review_and_apply():
    """
    Run Codex AI code review and apply feedback:
    1. Run workflow.codex_review() to get review results
    2. Analyze the review feedback
    3. Apply necessary fixes
    4. Re-run cargo build and cargo test to verify
    """
    workflow.codex_review()


@workflow.verify
def check_builds():
    """Project builds without errors."""
    workflow.run_command("cargo build 2>&1")


@workflow.verify
def check_tests():
    """All tests pass including existing cli tests."""
    workflow.run_command("cargo test 2>&1")


@workflow.post_job
def save_learnings():
    """Record learnings."""
    workflow.update_memory("learning", "profile-dispatch-v2", "Wired profile dispatch with manual match routing matching v1 pattern")


@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()
