#!/usr/bin/env python3
"""Task 001: Implement profile_default() with get/set/unset branching"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-001",
    version="v5",
    title="Implement profile_default() with get/set/unset branching",
    description=(
        "Add profile_default(args) function to src/profile.rs that handles three modes: "
        "get (no args → print current default), set (one name arg → validate + set), "
        "unset (--unset flag → clear default). Optionally add set_default()/clear_default() "
        "helper methods to CyoloConfig in src/config.rs."
    ),
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="high",
    estimated_hours=2,
    phase="Phase 1: Core Functions",
    dependencies=[],
    subtasks=[
        Subtask("task-001-1", "Implement profile_default() function in profile.rs", 1.5),
        Subtask("task-001-2", "Optionally add set_default/clear_default helpers to config.rs", 0.5),
    ],
    acceptance_criteria=[
        "profile_default([]) (no args) prints current default or 'No default profile set.'",
        "profile_default(['work']) with registered 'work' sets config.default to 'work' and saves",
        "profile_default(['unknown']) with unregistered name returns ProfileNotFound error",
        "profile_default(['--unset']) sets config.default to None and saves",
        "profile_default(['a', 'b']) (too many args) returns usage error",
        "cargo build succeeds",
        "cargo test passes",
    ],
    files=["src/profile.rs", "src/config.rs"],
    technical_notes=(
        "CyoloConfig.default is already Option<String>. Atomic save via CyoloConfig::save() "
        "is already implemented. Reuse ProfileNotFound error for invalid name validation. "
        "Use NonZeroExit(1) for usage errors. Pattern: load config, branch on arg count/value, "
        "mutate config.default, save."
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
def investigate_codebase():
    """
    Read existing code to understand patterns:
    - src/profile.rs: see dispatch(), add(), rm(), list() for arg-parsing conventions
    - src/config.rs: see CyoloConfig struct, load()/save() methods
    - src/error.rs: see ProfileNotFound, NonZeroExit(1) error types
    """
    pass

@workflow.step(2)
def implement_profile_default():
    """
    Implement profile_default(args: &[String]) in src/profile.rs:

    1. Match on args length and content:
       - No args → GET mode:
         - Load config, read config.default
         - If Some(name): println!("Default profile: {name}")
         - If None: println!("No default profile set.")
       - One arg == "--unset" → UNSET mode:
         - Load config, set config.default = None, save
         - println!("Default profile cleared.")
       - One arg (name) → SET mode:
         - Load config, validate name exists in config.profiles
         - If not found: return Err(ProfileNotFound { name })
         - Set config.default = Some(name.clone()), save
         - println!("Default profile set to: {name}")
       - More than one arg → usage error:
         - eprintln!("Usage: cyolo profile default [name | --unset]")
         - return Err(NonZeroExit(1))

    2. Call config::ensure_dir()? at the start of the function.

    3. Optionally add convenience methods to CyoloConfig in config.rs:
       - pub fn set_default(&mut self, name: String) { self.default = Some(name); }
       - pub fn clear_default(&mut self) { self.default = None; }
       These are optional; direct field access works fine too.
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
    workflow.update_memory("learning", "profile-default", "Implemented profile default get/set/unset in profile.rs")

@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()

if __name__ == "__main__":
    workflow.run_cli()
