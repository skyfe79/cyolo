#!/usr/bin/env python3
"""Task 001: Create diet.rs module skeleton with data structures and format_size helper"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-001",
    version="v6",
    title="Create diet.rs module skeleton with data structures and format_size helper",
    description="""Create the src/diet.rs module with all data structures needed for the diet command:
    DietOptions (parsed CLI args), OrphanedProject (project path + entry size),
    OrphanedSession (session folder path + total size), DietReport (aggregated analysis results).
    Also implement the format_size() helper for human-readable byte formatting (B, KB, MB, GB)
    and add unit tests for format_size().""",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="high",
    estimated_hours=2,
    phase="Phase 1: Foundation",
    dependencies=[],
    subtasks=[
        Subtask("task-001-1", "Define DietOptions struct (apply: bool)", 0.5),
        Subtask("task-001-2", "Define OrphanedProject struct (path: String, entry_size: u64)", 0.5),
        Subtask("task-001-3", "Define OrphanedSession struct (folder_path: PathBuf, total_size: u64)", 0.5),
        Subtask("task-001-4", "Define DietReport struct (orphaned_projects, orphaned_sessions, active_count, config_size, session_dir_size)", 0.5),
        Subtask("task-001-5", "Implement format_size(bytes: u64) -> String with B/KB/MB/GB formatting", 1),
        Subtask("task-001-6", "Unit tests for format_size: 0 B, 512 B, 1.5 KB, 1.4 MB, GB boundary", 1),
    ],
    acceptance_criteria=[
        "src/diet.rs exists with all four data structures",
        "format_size(0) returns '0 B'",
        "format_size(512) returns '512 B'",
        "format_size(1536) returns '1.5 KB'",
        "format_size(1_500_000) returns '1.4 MB'",
        "cargo test passes",
        "cargo build succeeds",
    ],
    files=["src/diet.rs", "src/main.rs"],
    technical_notes="""Add 'mod diet;' to main.rs. Use pub(crate) visibility for structs.
    format_size uses 1024-based divisions with one decimal place.
    DietReport should derive Debug. OrphanedProject and OrphanedSession should derive Debug.""",
    web_search=[],
)

workflow = TaskWorkflow(task)

@workflow.pre_job
def load_context():
    """Load project context and understand existing module patterns."""
    workflow.retrieve_memory("learning")
    workflow.read_skills()

@workflow.step(1)
def investigate():
    """
    Read existing source files to understand patterns:
    1. Read src/main.rs — understand module declaration pattern
    2. Read src/config.rs — understand struct/impl patterns used in codebase
    3. Read src/error.rs — understand error types available
    """
    pass

@workflow.step(2)
def implement_data_structures():
    """
    Create src/diet.rs with the following structures:

    1. DietOptions struct:
       - apply: bool (whether --apply was provided)

    2. OrphanedProject struct:
       - path: String (absolute filesystem path from ~/.claude.json key)
       - entry_size: u64 (serialized JSON length in bytes for this entry)

    3. OrphanedSession struct:
       - folder_path: PathBuf (full path to session folder in ~/.claude/projects/)
       - total_size: u64 (sum of all file sizes within the folder)

    4. DietReport struct:
       - orphaned_projects: Vec<OrphanedProject>
       - orphaned_sessions: Vec<OrphanedSession>
       - active_project_count: usize
       - config_file_size: u64
       - session_dir_total_size: u64
       - claude_home: PathBuf

    5. format_size(bytes: u64) -> String:
       - 0 -> "0 B"
       - < 1024 -> "N B"
       - < 1024^2 -> "N.N KB"
       - < 1024^3 -> "N.N MB"
       - >= 1024^3 -> "N.N GB"
    """
    pass

@workflow.step(3)
def add_tests():
    """
    Add #[cfg(test)] mod tests to diet.rs with tests for format_size:
    - test_format_size_zero: 0 -> "0 B"
    - test_format_size_bytes: 512 -> "512 B"
    - test_format_size_kb: 1536 -> "1.5 KB"
    - test_format_size_mb: 1_500_000 -> "1.4 MB"
    - test_format_size_gb: 1_500_000_000 -> "1.4 GB"
    - test_format_size_exact_boundary: 1024 -> "1.0 KB"

    Add 'mod diet;' to src/main.rs.
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
    workflow.update_memory("learning", "diet-data-structures", "Created diet.rs module with DietOptions, OrphanedProject, OrphanedSession, DietReport structs and format_size helper")

@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()

if __name__ == "__main__":
    workflow.run_cli()
