#!/usr/bin/env python3
"""Task 005: Implement profile list command"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-005",
    version="v2",
    title="Implement profile list command",
    description="Add the profile list subcommand to src/profile.rs. Loads config and displays all profiles with aligned columns. Marks the default profile with '*' prefix. Shows helpful message if no profiles registered.",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="medium",
    estimated_hours=1.5,
    phase="Phase 2: Profile Commands",
    dependencies=["task-002"],
    subtasks=[
        Subtask("task-005-1", "Implement list() function in profile.rs", 0.5),
        Subtask("task-005-2", "Handle empty profiles with helpful message", 0.3),
        Subtask("task-005-3", "Implement column alignment with default marker", 0.7),
    ],
    acceptance_criteria=[
        "Empty config prints 'No profiles registered. Run: cyolo profile add <name>'",
        "Multiple profiles displayed with aligned names padded to longest",
        "Default profile marked with '* ' prefix, others with '  ' prefix",
        "Output format: '* name    -> /path/to/dir'",
        "cargo build succeeds",
    ],
    files=["src/profile.rs"],
    technical_notes="pub fn list() -> Result<(), CyoloError>. Load config, if profiles is empty print help message. Otherwise find max name length for padding, iterate profiles in BTreeMap order (already sorted), format with '*' or ' ' prefix based on config.default.",
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
    Read src/profile.rs to understand the existing functions.
    Read src/config.rs for CyoloConfig fields (default, profiles).
    """
    pass


@workflow.step(2)
def implement_profile_list():
    """
    Add to src/profile.rs:

    pub fn list() -> Result<(), CyoloError>
    1. config::ensure_dir()
    2. Load config
    3. If config.profiles.is_empty():
       println!("No profiles registered. Run: cyolo profile add <name>")
       return Ok(())
    4. Find max_width = max name length across all profiles
    5. For each (name, profile) in config.profiles (BTreeMap gives sorted order):
       - marker = if config.default == Some(name) { "* " } else { "  " }
       - println!("{marker}{name:<max_width$} -> {config_dir}")
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
    workflow.update_memory("learning", "profile-list-v2", "Implemented aligned profile list with default marker")


@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()
