#!/usr/bin/env python3
"""Task 004: Implement profile rm command"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-004",
    version="v2",
    title="Implement profile rm command",
    description="Add the profile rm subcommand to src/profile.rs. Loads config, validates profile exists, removes from config.profiles, clears default if removing the default profile, and atomic-saves config. Directory is preserved.",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="medium",
    estimated_hours=1.5,
    phase="Phase 2: Profile Commands",
    dependencies=["task-002"],
    subtasks=[
        Subtask("task-004-1", "Implement rm() function in profile.rs", 0.5),
        Subtask("task-004-2", "Handle default profile clearing on removal", 0.5),
        Subtask("task-004-3", "Print removal confirmation with directory preservation note", 0.5),
    ],
    acceptance_criteria=[
        "'cyolo profile rm work' removes profile from config",
        "Removing the default profile clears config.default to None",
        "'cyolo profile rm nonexistent' returns ProfileNotFound with suggestion",
        "Directory on disk is preserved (not deleted)",
        "Prints 'Removed profile: <name>' and 'Directory preserved: <path>'",
        "cargo build succeeds",
    ],
    files=["src/profile.rs"],
    technical_notes="pub fn rm(args: &[String]) -> Result<(), CyoloError>. Parse args[0] as name. Load config, check profiles.contains_key(&name), remove_entry, check if default == Some(name) and clear it, save, print confirmation.",
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
    Read src/profile.rs to understand the existing add() implementation pattern.
    Read src/config.rs for the CyoloConfig API.
    """
    pass


@workflow.step(2)
def implement_profile_rm():
    """
    Add to src/profile.rs:

    pub fn rm(args: &[String]) -> Result<(), CyoloError>
    1. Parse name from args[0] (required, error if missing)
    2. config::ensure_dir()
    3. Load config
    4. Check name exists in config.profiles -> ProfileNotFound if not
    5. Get config_dir before removing (for the print message)
    6. Remove from config.profiles
    7. If config.default == Some(name): set config.default = None
    8. config.save()
    9. println!("Removed profile: {name}")
    10. println!("Directory preserved: {config_dir}")
    """
    pass


@workflow.step(3)
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


@workflow.post_job
def save_learnings():
    """Record learnings."""
    workflow.update_memory("learning", "profile-rm-v2", "Implemented profile rm with default clearing")


@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()
