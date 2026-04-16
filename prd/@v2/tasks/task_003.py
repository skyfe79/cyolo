#!/usr/bin/env python3
"""Task 003: Implement profile add command"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-003",
    version="v2",
    title="Implement profile add command",
    description="Create the profile add subcommand in src/profile.rs. Loads config, validates name uniqueness, resolves config_dir (default to ~/.claude-<name>), expands tilde, creates directory with 0700, registers profile, and atomic-saves config.",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="medium",
    estimated_hours=2,
    phase="Phase 2: Profile Commands",
    dependencies=["task-002"],
    subtasks=[
        Subtask("task-003-1", "Create src/profile.rs with add() function", 0.5),
        Subtask("task-003-2", "Implement tilde expansion for config_dir argument", 0.5),
        Subtask("task-003-3", "Implement directory creation with 0700 permissions", 0.5),
        Subtask("task-003-4", "Register profile in config and atomic save", 0.5),
    ],
    acceptance_criteria=[
        "'cyolo profile add work' creates ~/.claude-work/ and registers in config",
        "'cyolo profile add personal ~/.claude' registers existing dir without creating",
        "'cyolo profile add work' twice returns ProfileAlreadyExists error",
        "Omitted config-dir defaults to ~/.claude-<name>",
        "Tilde in quoted path is expanded via dirs::home_dir()",
        "Prints confirmation: 'Added profile: <name> -> <config_dir>'",
        "cargo build succeeds",
    ],
    files=["src/profile.rs", "src/main.rs"],
    technical_notes="Start by creating src/profile.rs with pub fn add(args: &[String]) -> Result<(), CyoloError>. Parse args[0] as name (required), args[1] as optional config_dir. Use config::ensure_dir(), config::CyoloConfig::load(), then save(). Use DirBuilder with mode 0o700 for profile dir creation. Add `mod profile;` to main.rs.",
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
    Read src/config.rs to understand the CyoloConfig API (load, save, ensure_dir).
    Read src/error.rs to know available error variants.
    """
    pass


@workflow.step(2)
def implement_profile_add():
    """
    Create src/profile.rs with:

    1. pub fn add(args: &[String]) -> Result<(), CyoloError>
       - Parse args: name = args[0] (required, error if missing)
       - config_dir = args[1] if provided, else ~/.claude-<name>
       - Expand tilde: if config_dir starts with '~/', replace with home_dir
       - config::ensure_dir() to ensure ~/.cyolo/ exists
       - Load config via CyoloConfig::load()
       - Check name not in config.profiles -> ProfileAlreadyExists if duplicate
       - Create config_dir with DirBuilder mode 0o700 if it doesn't exist
       - Insert Profile { name, config_dir } into config.profiles
       - config.save()
       - println!("Added profile: {name} -> {config_dir}")

    2. Add `mod profile;` to src/main.rs
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
    workflow.update_memory("learning", "profile-add-v2", "Implemented profile add with tilde expansion and 0700 dir creation")


@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()
