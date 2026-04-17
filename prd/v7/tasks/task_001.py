#!/usr/bin/env python3
"""Task 001: Implement is_claude_running() process detection"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-001",
    version="v7",
    title="Implement is_claude_running() process detection",
    description="""Add is_claude_running() function to diet.rs that detects whether a Claude Code
    process is currently running via pgrep -f "claude". Returns bool. Uses
    std::process::Command with unwrap_or(false) fallback when pgrep is unavailable.""",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="high",
    estimated_hours=1,
    phase="Phase 1: Safety Primitives",
    dependencies=[],
    subtasks=[
        Subtask("task-001-1", "Implement is_claude_running() using std::process::Command + pgrep", 0.5),
        Subtask("task-001-2", "Unit tests for is_claude_running() (compiles, returns bool)", 0.5),
    ],
    acceptance_criteria=[
        "is_claude_running() exists in src/diet.rs and compiles",
        "Function returns bool",
        "Returns false when pgrep is not available (unwrap_or fallback)",
        "cargo test passes",
        "cargo build succeeds",
    ],
    files=["src/diet.rs"],
    technical_notes="""Use std::process::Command::new("pgrep").args(["-f", "claude"]).output().
    Exit code 0 = running, 1 = not running. If Command::output() returns Err, return false.
    Function signature: pub(crate) fn is_claude_running() -> bool.
    No new dependencies needed — std::process::Command is in stdlib.""",
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
    Read existing source files to understand patterns:
    1. Read src/diet.rs — understand current structure and where to add the function
    2. Read src/error.rs — understand error types (not needed for this function since it returns bool)
    """
    pass

@workflow.step(2)
def implement_process_detection():
    """
    Add is_claude_running() to src/diet.rs:

    1. Add `use std::process::Command;` import (if not already present)
    2. Implement the function:
       ```rust
       pub(crate) fn is_claude_running() -> bool {
           Command::new("pgrep")
               .args(["-f", "claude"])
               .output()
               .map(|o| o.status.success())
               .unwrap_or(false)
       }
       ```
    3. Place it after the existing imports, before the struct definitions
    """
    pass

@workflow.step(3)
def add_tests():
    """
    Add unit tests in the existing #[cfg(test)] mod tests block:
    - test_is_claude_running_returns_bool: call the function, verify it returns without panic
      (cannot deterministically test true/false since we can't control pgrep in unit tests)

    Note: The function's behavior depends on runtime state (whether claude is running).
    Test that it compiles and doesn't panic. Integration testing is out of scope.
    """
    pass

@workflow.step(4)
def codex_review_and_apply():
    """
    Run Codex AI code review and apply feedback:
    1. Run workflow.codex_review() to get review results
    2. Analyze the review feedback (code quality, security, performance issues)
    3. Apply necessary fixes based on the review
    4. Re-run build/test to ensure fixes don't break anything
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
    workflow.update_memory("learning", "process-detection", "Implemented is_claude_running() via pgrep with unwrap_or(false) fallback")

@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()

if __name__ == "__main__":
    workflow.run_cli()
