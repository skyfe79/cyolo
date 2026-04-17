#!/usr/bin/env python3
"""Task 005: Wire detect::resolve_profile() into cli::route() for Command::Claude"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-005",
    version="v4",
    title="Wire detect::resolve_profile() into cli::route() for Command::Claude",
    description="Modify cli.rs route() function: for Command::Claude, call detect::resolve_profile() before runner::run_claude(). Pass the resolved config_dir (or None) to the runner. Profile, Diet, and Update commands must NOT trigger detection.",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="high",
    estimated_hours=1,
    phase="Phase 3: Integration",
    dependencies=["task-003", "task-004"],
    subtasks=[
        Subtask("task-005-1", "Call detect::resolve_profile() in Command::Claude branch", 0.4),
        Subtask("task-005-2", "Extract config_dir from ResolvedProfile and pass to run_claude()", 0.3),
        Subtask("task-005-3", "Verify Update and Profile commands do NOT call detection", 0.3),
    ],
    acceptance_criteria=[
        "Command::Claude calls detect::resolve_profile() before run_claude()",
        "Resolved config_dir is passed to run_claude() (Some or None)",
        "Command::Update does NOT trigger detection, calls run_update() directly",
        "Command::Profile does NOT trigger detection",
        "Detection errors (malformed JSON, unregistered name) propagate as CyoloError",
        "cargo build succeeds with no warnings",
    ],
    files=["src/cli.rs"],
    technical_notes="In the Command::Claude match arm, replace the current run_claude(&args, None) with: let resolved = detect::resolve_profile()?; let config_dir = resolved.as_ref().map(|r| r.config_dir.as_path()); runner::run_claude(&args, config_dir)?;",
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
    Read src/cli.rs to see the route() function and its match arms.
    Read src/detect.rs to confirm resolve_profile() signature.
    Read src/runner.rs to confirm run_claude() signature after task-004.
    """
    pass


@workflow.step(2)
def wire_detection():
    """
    Modify src/cli.rs route() function:

    1. Add `use crate::detect;` at the top.

    2. In the Command::Claude match arm:
       let resolved = detect::resolve_profile()?;
       let config_dir = resolved.as_ref().map(|r| r.config_dir.as_path());
       runner::run_claude(&args, config_dir)?;

    3. Verify Command::Update arm still calls runner::run_update() directly.
    4. Verify Command::Profile arm still calls profile::dispatch() directly.
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
    workflow.update_memory("learning", "cli-detection-v4", "Wired detect::resolve_profile() into cli::route() for Command::Claude only")


@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()
