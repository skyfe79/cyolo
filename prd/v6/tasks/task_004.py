#!/usr/bin/env python3
"""Task 004: Implement print_report() — tree-style dry-run report"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-004",
    version="v6",
    title="Implement print_report() for tree-style dry-run report output",
    description="""Print a tree-style report of the diet analysis results using Unicode
    box-drawing characters. Show orphaned projects (up to 5, collapse remainder),
    active project count, orphaned session folders, and total reclaimable size.
    Adapt footer text based on --apply mode.""",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="medium",
    estimated_hours=2,
    phase="Phase 3: Output",
    dependencies=["task-001"],
    subtasks=[
        Subtask("task-004-1", "Implement print_report(report: &DietReport, applied: bool) function", 1),
        Subtask("task-004-2", "Handle edge cases: 0 orphans, 1 orphan, 5+, 10+ orphans", 0.5),
        Subtask("task-004-3", "Unit tests verifying output format for various scenarios", 1),
    ],
    acceptance_criteria=[
        "Tree-style output uses Unicode box-drawing characters",
        "Shows up to 5 orphaned paths; collapses remainder with '... N more'",
        "When applied=false: shows 'Run with --apply to proceed.'",
        "When applied=true: shows 'Cleanup complete.'",
        "When no orphans: shows 'No orphaned projects found. Nothing to clean up.'",
        "Size values use format_size() helper",
        "cargo test passes",
    ],
    files=["src/diet.rs"],
    technical_notes="""Output format from PRD:

    cyolo diet — analyzing ~/.claude

    ~/.claude.json:                          N.N MB
      orphaned projects (N):              N.N KB  (removable)
         /path/to/deleted/project1       N.N KB
         /path/to/deleted/project2       N.N KB
         /path/to/deleted/project3       N.N KB
      active projects (N):               N.N KB  (keep)

    ~/.claude/projects/:                     N.N MB
      orphaned session folders (N):       N.N MB  (removable)

    Total reclaimable: N.N MB

    Run with --apply to proceed.

    Use eprintln! or println! for output. Return String for testability.""",
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
    Read src/diet.rs to understand DietReport struct and format_size().
    Review PRD F5 section for exact output format specification.
    """
    pass

@workflow.step(2)
def implement_report():
    """
    Implement two functions:

    1. build_report_string(report: &DietReport, applied: bool) -> String:
       - Header: "cyolo diet — analyzing {claude_home display path}"
       - ~/.claude.json section with config_file_size
       - Orphaned projects subsection (up to 5 paths listed)
       - Active projects subsection
       - ~/.claude/projects/ section with session_dir_total_size
       - Orphaned session folders subsection
       - Total reclaimable line (sum of orphaned project sizes + orphaned session sizes)
       - Footer: "Run with --apply to proceed." or "Cleanup complete."
       - Special case: no orphans -> "No orphaned projects found. Nothing to clean up."

    2. print_report(report: &DietReport, applied: bool):
       - Calls build_report_string and prints via println!

    Use Unicode box-drawing: \u2502 (│), \u251c\u2500 (├─), \u2514\u2500 (└─)
    Tilde shorthand: replace home_dir prefix with ~ in display paths.
    """
    pass

@workflow.step(3)
def add_tests():
    """
    Add unit tests for build_report_string:
    - test_report_no_orphans: empty orphans -> "No orphaned projects found" message
    - test_report_one_orphan: single orphaned project -> listed directly
    - test_report_five_orphans: exactly 5 -> all listed, no "... more"
    - test_report_ten_orphans: 10 -> first 5 listed + "... 5 more"
    - test_report_applied_footer: applied=true -> "Cleanup complete."
    - test_report_dry_run_footer: applied=false -> "Run with --apply"
    """
    pass

@workflow.step(4)
def codex_review_and_apply():
    """
    Run Codex AI code review and apply feedback:
    1. Run workflow.codex_review() to get review results
    2. Analyze the review feedback
    3. Apply necessary fixes
    4. Re-run cargo build/test to verify
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
    workflow.update_memory("learning", "diet-report", "Implemented tree-style report with Unicode box-drawing and tilde path shorthand")

@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()

if __name__ == "__main__":
    workflow.run_cli()
