#!/usr/bin/env python3
"""Task 006: Handle edge cases and improve error messages"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-006",
    version="v9",
    title="Handle edge cases and improve error messages",
    description="""Review and improve error messages across the codebase after
coloring has been applied. Ensure edge cases are handled:
- JSON parse errors in diet.rs and profile.rs show the file path clearly
- ProfileNotFound displays correctly through the colored error handler
- Add #[cfg(test)] helper to disable colors in tests so string assertions work
- Review all error paths for clarity and consistency""",
    status="pending",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="medium",
    estimated_hours=4,
    phase="Phase 4: Edge Cases & Polish",
    dependencies=["task-002", "task-003", "task-004", "task-005"],
    subtasks=[
        Subtask("task-006-1", "Review and improve JSON parse error messages with file paths", 1),
        Subtask("task-006-2", "Verify ProfileNotFound renders correctly through colored handler", 0.5),
        Subtask("task-006-3", "Add #[cfg(test)] setup to disable colors in test modules", 1),
        Subtask("task-006-4", "Review all error messages for consistency and clarity", 1),
        Subtask("task-006-5", "Fix any tests broken by ANSI codes in string comparisons", 0.5),
    ],
    acceptance_criteria=[
        "JSON parse errors in diet.rs include the file path being parsed",
        "JSON parse errors in profile.rs include the file path being parsed",
        "ProfileNotFound error displays profile name through the colored error handler",
        "A #[cfg(test)] block calls owo_colors::set_override(false) in test setup",
        "All existing tests pass without ANSI code interference",
        "Error messages are consistent: colored prefix + clear description + context",
        "No error message loses information compared to pre-v9 versions",
        "cargo build succeeds",
        "cargo test passes",
    ],
    files=["src/diet.rs", "src/profile.rs", "src/error.rs"],
    technical_notes="""For JSON parse error improvements:
- When parsing JSON fails, include the file path in the error message
- Pattern: eprintln!("{} failed to parse {}: {}", "error:".red().bold(), path.display(), err);

For ProfileNotFound verification:
- Trigger the error path manually to confirm it flows through main.rs error handler
- The colored "error:" prefix in main.rs should render before the ProfileNotFound message

For #[cfg(test)] color disable:
- In each module with tests, add to the test module:
  #[cfg(test)]
  mod tests {
      use super::*;

      fn setup() {
          owo_colors::set_override(false);
      }
      // ... existing tests, each calling setup() at start
  }

- Alternatively, use a test fixture / ctor pattern. The simplest approach
  is to call set_override(false) at the beginning of each test function
  that compares string output.

For consistency review:
- Grep all eprintln!/println! across modified files
- Ensure every error has "error:" red bold prefix
- Ensure every warning has "warning:" yellow bold prefix
- Ensure no message lost its context (file path, profile name, etc.)""",
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
    Audit all error messages across the codebase:
    1. Read src/diet.rs — find all JSON parsing code and error messages
    2. Read src/profile.rs — find all JSON parsing code and error messages
    3. Read src/error.rs — review ProfileNotFound and its Display impl
    4. Read src/main.rs — verify the colored error handler from task-002
    5. Grep for eprintln! across all .rs files to get a complete picture
    6. Run cargo test to identify any tests failing due to ANSI codes
    """
    pass

@workflow.step(2)
def improve_json_error_messages():
    """
    Ensure JSON parse errors include file paths:
    1. In diet.rs: find serde_json::from_str or from_reader calls
       - Ensure the error message includes the file path: "failed to parse {path}: {err}"
    2. In profile.rs: find serde_json::from_str or from_reader calls
       - Ensure the error message includes the file path: "failed to parse {path}: {err}"
    3. Use the colored prefix consistently: "error:".red().bold()
    """
    pass

@workflow.step(3)
def verify_profile_not_found():
    """
    Verify ProfileNotFound displays correctly:
    1. Check the Display impl for ProfileNotFound in error.rs
    2. Trace the error flow from profile.rs → main.rs error handler
    3. Ensure the profile name is included in the error message
    4. Confirm the main.rs colored "error:" prefix renders before the message
    5. If needed, adjust the Display impl to be more descriptive
    """
    pass

@workflow.step(4)
def add_test_color_disable():
    """
    Add #[cfg(test)] color disable setup:
    1. In diet.rs test module: add owo_colors::set_override(false) call
    2. In profile.rs test module: add owo_colors::set_override(false) call
    3. In any other test module that compares string output: add the call
    4. Place the call at the start of each test function that does string comparison
    5. Run cargo test to verify all tests pass
    """
    pass

@workflow.step(5)
def consistency_review():
    """
    Final consistency review of all error messages:
    1. Grep all eprintln!/println! in modified files
    2. Verify every error → "error:".red().bold()
    3. Verify every warning → "warning:".yellow().bold()
    4. Verify every success → green or checkmark
    5. Check no error message lost context compared to v8
    6. Ensure usage messages still show complete help text
    """
    pass

@workflow.step(6)
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
    workflow.update_memory("learning", "v9-task-006", "Added test color disable, improved JSON error messages with file paths")

@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()

if __name__ == "__main__":
    workflow.run_cli()
