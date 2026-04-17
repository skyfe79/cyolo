#!/usr/bin/env python3
"""Task 002: Color error output in main.rs and clean up error variants"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-002",
    version="v9",
    title="Color error output in main.rs and clean up error variants",
    description="""Apply colored formatting to the top-level error handler in main.rs
so that "error:" appears in red bold when printing CyoloError.

Also clean up src/error.rs by removing dead/unused variants:
- Remove #[allow(dead_code)] from ProfileAlreadyExists, ProfileNotFound, ProfileFileError
  (these are now actively used and should not have dead_code suppression)
- Remove the SymlinkError variant entirely (it is never constructed anywhere)""",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="high",
    estimated_hours=3,
    phase="Phase 2: Core Error Coloring",
    dependencies=["task-001"],
    subtasks=[
        Subtask("task-002-1", "Color the error prefix in main.rs error handler", 1),
        Subtask("task-002-2", "Remove #[allow(dead_code)] from actively-used error variants", 1),
        Subtask("task-002-3", "Remove SymlinkError variant from CyoloError enum", 1),
    ],
    acceptance_criteria=[
        "main.rs error handler prints 'error:' in red bold using OwoColorize",
        "Error message body prints after the colored prefix",
        "#[allow(dead_code)] removed from ProfileAlreadyExists variant",
        "#[allow(dead_code)] removed from ProfileNotFound variant",
        "#[allow(dead_code)] removed from ProfileFileError variant",
        "SymlinkError variant is completely removed from CyoloError enum",
        "No references to SymlinkError remain in any source file",
        "cargo build succeeds with no warnings about dead_code on the three variants",
        "cargo test passes",
    ],
    files=["src/main.rs", "src/error.rs"],
    technical_notes="""In main.rs, the error handler likely does something like:
eprintln!("error: {}", e);

Change to use OwoColorize:
use owo_colors::OwoColorize;
eprintln!("{} {}", "error:".red().bold(), e);

In error.rs, find the CyoloError enum and:
1. Remove any #[allow(dead_code)] annotations on ProfileAlreadyExists,
   ProfileNotFound, and ProfileFileError variants
2. Delete the SymlinkError variant and its Display/From implementations
3. Search all .rs files to confirm SymlinkError is never referenced""",
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
    Read source files to understand current error handling:
    1. Read src/main.rs — find the error handler (match on Result or if let Err)
    2. Read src/error.rs — catalog all CyoloError variants and their annotations
    3. Grep all .rs files for 'SymlinkError' to confirm it is never constructed
    4. Grep for '#[allow(dead_code)]' to find all suppression annotations
    """
    pass

@workflow.step(2)
def color_error_output():
    """
    Add colored error prefix in main.rs:
    1. Add import: use owo_colors::OwoColorize;
    2. Find the error printing code (eprintln with error message)
    3. Change to: eprintln!("{} {}", "error:".red().bold(), e);
    4. Ensure the format preserves the full error message
    """
    pass

@workflow.step(3)
def clean_error_variants():
    """
    Clean up error.rs:
    1. Remove #[allow(dead_code)] from ProfileAlreadyExists variant
    2. Remove #[allow(dead_code)] from ProfileNotFound variant
    3. Remove #[allow(dead_code)] from ProfileFileError variant
    4. Remove the entire SymlinkError variant definition
    5. Remove SymlinkError arm from the Display impl (if present)
    6. Remove any From impl for SymlinkError (if present)
    7. Verify no other file references SymlinkError
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
    workflow.update_memory("learning", "v9-task-002", "Colored error handler in main.rs and removed dead SymlinkError variant")

@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()

if __name__ == "__main__":
    workflow.run_cli()
