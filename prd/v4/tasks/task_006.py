#!/usr/bin/env python3
"""Task 006: Add profile current subcommand"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-006",
    version="v4",
    title="Add profile current subcommand",
    description="Add a 'profile current' subcommand that runs detect::resolve_profile() and prints the result. Shows profile name, config_dir, and source when detected. Shows informational message when no profile detected. Does NOT launch claude.",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="medium",
    estimated_hours=1,
    phase="Phase 3: Integration",
    dependencies=["task-003"],
    subtasks=[
        Subtask("task-006-1", "Add 'current' to profile subcommand parsing in cli.rs or profile.rs", 0.3),
        Subtask("task-006-2", "Implement current display logic with profile info output", 0.4),
        Subtask("task-006-3", "Handle no-profile-detected case with informational message", 0.3),
    ],
    acceptance_criteria=[
        "cyolo profile current runs detect::resolve_profile() and prints result",
        "When profile detected: shows profile name, config_dir, and source file path",
        "When using default: shows 'default' as source",
        "When no profile detected: shows 'No profile detected. Using default Claude configuration (~/.claude).'",
        "Does NOT launch claude",
        "cargo build succeeds with no warnings",
    ],
    files=["src/profile.rs", "src/cli.rs"],
    technical_notes="The profile dispatch function in profile.rs handles subcommands (add, rm, list, link). Add 'current' as a new match arm. Call detect::resolve_profile() and format the output. Output format: 'profile: {name}\\nconfig_dir: {dir}\\nsource: {source}' when detected, informational message otherwise. Update help text in dispatch to include 'current'.",
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
    Read src/profile.rs to understand the dispatch function and existing subcommands.
    Read src/cli.rs to see how profile subcommands are routed.
    Read src/detect.rs to confirm resolve_profile() return type.
    """
    pass


@workflow.step(2)
def implement_current_subcommand():
    """
    In src/profile.rs:

    1. Add "current" match arm in dispatch():
       "current" => {
           let resolved = crate::detect::resolve_profile()?;
           match resolved {
               Some(profile) => {
                   if let Some(name) = &profile.name {
                       println!("profile: {}", name);
                   }
                   println!("config_dir: {}", profile.config_dir.display());
                   println!("source: {}", profile.source);
               }
               None => {
                   println!("No profile detected. Using default Claude configuration (~/.claude).");
               }
           }
       }

    2. Update the help/usage text to include "current" subcommand.
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
    workflow.update_memory("learning", "profile-current-v4", "Added profile current subcommand to display detected profile info")


@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()
