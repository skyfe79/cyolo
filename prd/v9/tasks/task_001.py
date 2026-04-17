#!/usr/bin/env python3
"""Task 001: Add owo-colors dependency and terminal detection"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-001",
    version="v9",
    title="Add owo-colors dependency and terminal detection",
    description="""Add the owo-colors crate with supports-color feature to Cargo.toml
and configure terminal detection in main.rs so that ANSI color codes
are suppressed when stderr is not a TTY (piped output, CI, etc.).

This is the foundation task that all other v9 color tasks depend on.""",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="high",
    estimated_hours=2,
    phase="Phase 1: Color Foundation",
    dependencies=[],
    subtasks=[
        Subtask("task-001-1", "Add owo-colors dependency to Cargo.toml", 0.5),
        Subtask("task-001-2", "Import and configure terminal detection in main.rs", 1),
        Subtask("task-001-3", "Verify cargo build succeeds with new dependency", 0.5),
    ],
    acceptance_criteria=[
        "Cargo.toml contains owo-colors = { version = \"4\", features = [\"supports-colors\"] } (v5 yanked, set_override requires supports-colors)",
        "main.rs imports owo_colors::set_override and std::io::IsTerminal",
        "main() calls set_override(false) when std::io::stderr() is not a terminal",
        "When stderr is a TTY, colors remain enabled (no override needed, owo-colors auto-detects)",
        "cargo build succeeds",
        "cargo test passes",
    ],
    files=["Cargo.toml", "src/main.rs"],
    technical_notes="""In main.rs, add at the top:
use owo_colors::set_override;
use std::io::IsTerminal;

In fn main(), before any other logic:
if !std::io::stderr().is_terminal() {
    set_override(false);
}

This ensures piped/redirected output never contains ANSI escape codes.
The set_override(false) call globally disables owo-colors output.""",
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
    Read current files to understand the starting point:
    1. Read Cargo.toml to see existing dependencies
    2. Read src/main.rs to understand the main() function structure
    3. Note where imports are placed and where early initialization happens
    """
    pass

@workflow.step(2)
def add_dependency():
    """
    Add owo-colors to Cargo.toml [dependencies] section:
    1. Add line: owo-colors = { version = "5", features = ["supports-color"] }
    2. Place it alphabetically among existing dependencies
    """
    pass

@workflow.step(3)
def implement_terminal_detection():
    """
    Configure terminal detection in src/main.rs:
    1. Add import: use owo_colors::set_override;
    2. Add import: use std::io::IsTerminal;
    3. At the start of fn main(), before any CLI parsing or logic, add:
       if !std::io::stderr().is_terminal() {
           set_override(false);
       }
    4. This suppresses color output when stderr is piped/redirected
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
    workflow.update_memory("learning", "v9-task-001", "Added owo-colors with terminal detection for conditional color output")

@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()

if __name__ == "__main__":
    workflow.run_cli()
