#!/usr/bin/env python3
"""Task 004: Color warnings in symlink.rs and detect.rs"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-004",
    version="v9",
    title="Color warnings in symlink.rs and detect.rs",
    description="""Add colored output to symlink.rs and detect.rs:
- symlink.rs: "warning:" in yellow bold, "error:" in red bold for failures
- detect.rs: "warning:" in yellow bold
- Drop the "cyolo:" prefix from messages in favor of the colored severity labels""",
    status="pending",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="medium",
    estimated_hours=3,
    phase="Phase 3: Module Coloring",
    dependencies=["task-001"],
    subtasks=[
        Subtask("task-004-1", "Import OwoColorize in symlink.rs and detect.rs", 0.5),
        Subtask("task-004-2", "Color warning and error messages in symlink.rs", 1),
        Subtask("task-004-3", "Color warning messages in detect.rs", 0.5),
        Subtask("task-004-4", "Remove cyolo: prefix from messages in both files", 1),
    ],
    acceptance_criteria=[
        "symlink.rs imports owo_colors::OwoColorize",
        "detect.rs imports owo_colors::OwoColorize",
        "symlink.rs warning messages use 'warning:'.yellow().bold() prefix",
        "symlink.rs error messages use 'error:'.red().bold() prefix",
        "detect.rs warning messages use 'warning:'.yellow().bold() prefix",
        "No messages in symlink.rs or detect.rs start with 'cyolo:' prefix",
        "All colored prefixes follow the pattern: severity label + colon, colored",
        "cargo build succeeds",
        "cargo test passes",
    ],
    files=["src/symlink.rs", "src/detect.rs"],
    technical_notes="""Import in both files:
use owo_colors::OwoColorize;

In symlink.rs, find all eprintln! calls:
- Messages about symlink creation failures → "error:".red().bold()
- Messages about missing targets or warnings → "warning:".yellow().bold()
- Replace any "cyolo: " prefix with the colored severity label

In detect.rs, find all eprintln! calls:
- Warning messages about detection issues → "warning:".yellow().bold()
- Replace any "cyolo: " prefix with the colored severity label

Pattern:
  Before: eprintln!("cyolo: could not read symlink: {}", path);
  After:  eprintln!("{} could not read symlink: {}", "warning:".yellow().bold(), path);""",
    web_search=[],
)

workflow = TaskWorkflow(task)

@workflow.pre_job
def load_context():
    """Load project context and skill files."""
    workflow.retrieve_memory("learning")
    workflow.read_skills()

@workflow.step(1)
def investigate():
    """
    Read both source files and catalog all user-facing output:
    1. Read src/symlink.rs — find all eprintln!/println! calls
    2. Read src/detect.rs — find all eprintln!/println! calls
    3. Identify which messages are errors vs warnings
    4. Note any "cyolo:" prefixes that need removal
    """
    pass

@workflow.step(2)
def color_symlink_messages():
    """
    Add colored output to symlink.rs:
    1. Add import: use owo_colors::OwoColorize;
    2. Find all error eprintln! → prefix with "error:".red().bold()
    3. Find all warning eprintln! → prefix with "warning:".yellow().bold()
    4. Remove any "cyolo:" prefix from messages
    5. Ensure message body text remains uncolored for readability
    """
    pass

@workflow.step(3)
def color_detect_messages():
    """
    Add colored output to detect.rs:
    1. Add import: use owo_colors::OwoColorize;
    2. Find all warning eprintln! → prefix with "warning:".yellow().bold()
    3. Remove any "cyolo:" prefix from messages
    4. Ensure message body text remains uncolored for readability
    """
    pass

@workflow.step(4)
def codex_review_and_apply():
    """
    Run Codex AI code review and apply feedback:
    1. Run workflow.codex_review() to get review results
    2. Analyze the review feedback
    3. Apply necessary fixes
    4. Re-run cargo build && cargo test to verify fixes
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
    """Record learnings."""
    workflow.update_memory("learning", "v9-task-004", "Colored warnings/errors in symlink.rs and detect.rs, removed cyolo: prefix")

@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()

if __name__ == "__main__":
    workflow.run_cli()
