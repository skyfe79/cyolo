#!/usr/bin/env python3
"""Task 006: Update profile dispatch and usage help for link command"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-006",
    version="v3",
    title="Update profile dispatch and usage help for link command",
    description="Add 'link' arm to the profile dispatch match and update the usage help text to include the link command and --no-share note for add.",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="medium",
    estimated_hours=0.5,
    phase="Phase 4: Polish",
    dependencies=["task-004", "task-005"],
    subtasks=[
        Subtask("task-006-1", "Add 'link' arm to dispatch match in profile.rs", 0.2),
        Subtask("task-006-2", "Update usage help text with link command and --no-share note", 0.3),
    ],
    acceptance_criteria=[
        "cyolo profile link work dispatches to profile::link correctly",
        "cyolo profile (no args) shows updated help including 'link' command",
        "Usage help mentions --no-share option for add command",
        "cargo build succeeds",
    ],
    files=["src/profile.rs"],
    technical_notes="""Two changes needed:
1. In the dispatch match (existing pattern: 'add' | 'rm' | 'list'), add 'link' arm calling profile::link(&rest)
2. In the usage/help text, add:
   - 'profile link <name>  Re-create shared symlinks'
   - Update 'profile add' line to mention '--no-share' option""",
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
    Read src/profile.rs to find the dispatch match and usage help text.
    Note the pattern used for existing subcommands (add, rm, list).
    """
    pass


@workflow.step(2)
def update_dispatch_and_help():
    """
    1. Find the dispatch match in profile.rs (likely in a run() or dispatch() function).
       Add: "link" => link(&rest_args),

    2. Find the usage help text (likely a println! block or help function).
       Update to include:
       - "  link <name>        Re-create shared symlinks for a profile"
       - Update the add line: "  add <name> [dir] [--no-share]  Register a new profile"
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
    workflow.update_memory("learning", "dispatch-link-v3", "Added link arm to profile dispatch and updated help text with --no-share docs")


@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()
