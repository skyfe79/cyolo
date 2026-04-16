#!/usr/bin/env python3
"""Task 005: Add profile link command"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-005",
    version="v3",
    title="Add profile link command",
    description="Implement 'profile link <name>' command that re-creates symlinks on an already-registered profile. Idempotent — existing correct symlinks are left as-is.",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="medium",
    estimated_hours=1,
    phase="Phase 3: Integration",
    dependencies=["task-003"],
    subtasks=[
        Subtask("task-005-1", "Implement link() function: load config, find profile, call create_shared_symlinks", 0.5),
        Subtask("task-005-2", "Handle ProfileNotFound error", 0.2),
        Subtask("task-005-3", "Print confirmation message", 0.2),
    ],
    acceptance_criteria=[
        "cyolo profile link work -> creates missing symlinks, skips existing ones",
        "cyolo profile link personal (where personal=~/.claude) -> self-reference skip message",
        "cyolo profile link unknown -> ProfileNotFound error",
        "Running link twice produces same result (idempotent)",
        "cargo build succeeds",
    ],
    files=["src/profile.rs"],
    technical_notes="""Implementation:
1. Load config via config::load()
2. Find profile by name in config.profiles -> ProfileNotFound if missing
3. Call symlink::create_shared_symlinks(&profile.config_dir)
4. Print confirmation: "Symlinks updated for profile '<name>'"

Idempotency comes from create_shared_symlinks: it uses symlink_metadata to detect existing targets and skips them with a warning.""",
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
    Read src/profile.rs to understand existing command patterns (add, rm, list).
    Read src/config.rs to understand how to load config and look up profiles.
    """
    pass


@workflow.step(2)
def implement_link():
    """
    Add pub fn link(args: &[String]) -> Result<(), CyoloError> to profile.rs:

    1. Validate args: require exactly one arg (profile name)
       - If no args: return error with usage hint
    2. Load config via config::load()?
    3. Look up profile by name in config.profiles
       - If not found: return CyoloError::ProfileNotFound { name }
    4. Resolve config_dir (expand tilde if needed)
    5. Call symlink::create_shared_symlinks(&config_dir)?
    6. println!("Symlinks updated for profile '{name}'")
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
    """Existing tests pass."""
    workflow.run_command("cargo test 2>&1")


@workflow.post_job
def save_learnings():
    """Record learnings."""
    workflow.update_memory("learning", "profile-link-v3", "Implemented profile link command delegating to create_shared_symlinks for idempotent symlink re-creation")


@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()
