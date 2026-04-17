#!/usr/bin/env python3
"""Task 004: Modify run_claude() to accept config_dir parameter"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-004",
    version="v4",
    title="Modify run_claude() to accept config_dir parameter",
    description="Change run_claude() signature to accept config_dir: Option<&Path>. When Some, set CLAUDE_CONFIG_DIR env var on the child Command. When None, do not set the var (claude uses default ~/.claude). Update the existing call site in cli.rs to pass None temporarily.",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="high",
    estimated_hours=1,
    phase="Phase 2: Core Logic",
    dependencies=[],
    subtasks=[
        Subtask("task-004-1", "Add config_dir: Option<&Path> parameter to run_claude()", 0.3),
        Subtask("task-004-2", "Set CLAUDE_CONFIG_DIR on Command when config_dir is Some", 0.3),
        Subtask("task-004-3", "Update call site in cli.rs to pass None", 0.2),
        Subtask("task-004-4", "Verify run_update() is unaffected", 0.2),
    ],
    acceptance_criteria=[
        "run_claude() accepts config_dir: Option<&Path> parameter",
        "When Some(path), child process Command has CLAUDE_CONFIG_DIR env var set",
        "When None, no CLAUDE_CONFIG_DIR is set (inherits parent env)",
        "Existing call site in cli.rs updated to pass None (compiles correctly)",
        "run_update() signature and behavior unchanged",
        "cargo build succeeds with no warnings",
    ],
    files=["src/runner.rs", "src/cli.rs"],
    technical_notes="Use Command::env('CLAUDE_CONFIG_DIR', path) to set the env var on the child process. Only call .env() when config_dir is Some — do not set it to empty string. The existing call site in cli.rs currently calls run_claude(args); change to run_claude(args, None). Task 005 will later replace None with the detected profile's config_dir.",
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
    Read src/runner.rs to see current run_claude() and run_update() signatures.
    Read src/cli.rs to find the call site for run_claude().
    """
    pass


@workflow.step(2)
def implement_config_dir_param():
    """
    Modify src/runner.rs:

    1. Change run_claude signature:
       - Before: pub fn run_claude(args: &[String]) -> Result<(), CyoloError>
       - After:  pub fn run_claude(args: &[String], config_dir: Option<&Path>) -> Result<(), CyoloError>

    2. Inside run_claude, after building the Command:
       if let Some(dir) = config_dir {
           cmd.env("CLAUDE_CONFIG_DIR", dir);
       }

    3. Add `use std::path::Path;` if not already imported.

    4. In src/cli.rs, update the call:
       - Before: runner::run_claude(&args)?
       - After:  runner::run_claude(&args, None)?
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
    workflow.update_memory("learning", "runner-config-dir-v4", "Modified run_claude() to accept config_dir param, sets CLAUDE_CONFIG_DIR on child process")


@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()
