#!/usr/bin/env python3
"""Task 005: Extend DietReport and build_report_string() with stale/cache sections"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-005",
    version="v8",
    title="Extend DietReport and build_report_string() with stale and cache sections",
    description="""Add stale_projects and cache_dirs fields to DietReport.
Update build_report_string() to render stale projects section (after orphans)
and cache directories section (after sessions).

Stale projects section format:
  stale projects (3):                          1.2 MB  (history clearable)
    ├─ ~/work/old-client      last activity: 120 days ago     340 KB
    └─ ~/tmp/experiment       last activity: 91 days ago      580 KB

Cache directories section format:
  ~/.claude/cache:                              45.2 MB
    └─ clearable cache dirs (2):               45.2 MB  (removable)
        ├─ statsig/                            12.3 MB
        └─ file-history/                       32.9 MB

Total reclaimable must sum orphan entries + orphan sessions + stale history +
stale sessions + cache sizes.

"No orphaned projects found" message should also check stale and cache are empty.""",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="medium",
    estimated_hours=4,
    phase="Phase 3: Reporting",
    dependencies=["task-002", "task-003"],
    subtasks=[
        Subtask("task-005-1", "Add stale_projects and cache_dirs fields to DietReport", 0.5),
        Subtask("task-005-2", "Implement stale projects section in build_report_string()", 1.5),
        Subtask("task-005-3", "Implement cache directories section in build_report_string()", 1),
        Subtask("task-005-4", "Update total reclaimable calculation and no-data message", 0.5),
        Subtask("task-005-5", "Write unit tests and update existing make_report helper", 0.5),
    ],
    acceptance_criteria=[
        "DietReport has stale_projects: Vec<StaleProject> and cache_dirs: Vec<CacheDir>",
        "Report includes stale projects section only when stale_projects is non-empty",
        "Report includes cache section only when cache_dirs is non-empty",
        "Total reclaimable sums orphans + orphan sessions + stale history + stale sessions + cache",
        "Last activity displayed as 'N days ago'",
        "Existing report format for orphans is unchanged",
        "'Nothing to clean up' only shown when all vectors are empty",
        "cargo test passes",
    ],
    files=["src/diet.rs"],
    technical_notes="""DietReport struct is at line ~52. Add two new fields.
build_report_string() is at line ~285. Add sections between existing sections.

For 'N days ago': last_activity_secs / 86400 gives days.

Update the make_report() test helper to include the new fields with default empty vecs.
All existing tests should still pass with the updated helper.

The 'nothing to clean up' check at line ~289 needs to also check stale_projects and cache_dirs.

For total reclaimable:
- stale history: sum of stale_projects[].history_size
- stale sessions: sum of stale_projects[].session_size
- cache: sum of cache_dirs[].size""",
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
    Read src/diet.rs and understand:
    1. DietReport struct (line ~52) — current fields
    2. build_report_string() (line ~285) — full report rendering logic
    3. make_report() test helper — how tests construct DietReport
    4. StaleProject and CacheDir structs (from task-002 and task-003)
    """
    pass

@workflow.step(2)
def extend_diet_report():
    """
    Add to DietReport struct:
    - stale_projects: Vec<StaleProject>
    - cache_dirs: Vec<CacheDir>

    Update all DietReport construction sites (dispatch, tests) to include new fields.
    """
    pass

@workflow.step(3)
def implement_stale_section():
    """
    Add stale projects section in build_report_string() after orphaned projects:
    1. Check if stale_projects is non-empty
    2. Calculate total stale history + session sizes
    3. Render header with count and total size
    4. Render each stale project with path, days ago, and size
    5. Use same tree formatting (├─, └─) as orphaned projects
    """
    pass

@workflow.step(4)
def implement_cache_section():
    """
    Add cache directories section in build_report_string() after sessions:
    1. Check if cache_dirs is non-empty
    2. Calculate total cache size
    3. Render header with claude_home path and total size
    4. Render each cache dir with name and size
    5. Use tree formatting
    """
    pass

@workflow.step(5)
def update_totals_and_empty_check():
    """
    1. Update total reclaimable to sum all categories
    2. Update 'nothing to clean up' check to include stale and cache emptiness
    3. Update make_report() helper with default empty stale_projects and cache_dirs
    4. Verify all existing tests still pass
    """
    pass

@workflow.step(6)
def add_tests():
    """
    Add unit tests:
    1. test_report_with_stale_projects: stale projects in report → stale section rendered
    2. test_report_with_cache_dirs: cache dirs in report → cache section rendered
    3. test_report_stale_and_cache: both present → both sections rendered
    4. test_report_total_reclaimable_all: verify total includes all categories
    5. test_report_nothing_to_clean_all_empty: no orphans + no stale + no cache → clean message
    6. test_report_days_ago_format: verify "N days ago" formatting
    """
    pass

@workflow.step(7)
def codex_review_and_apply():
    """
    Run Codex AI code review and apply feedback:
    1. Run workflow.codex_review() to get review results
    2. Analyze the review feedback
    3. Apply necessary fixes
    4. Re-run cargo build && cargo test
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
    workflow.update_memory("learning", "v8-task-005", "Extended report with stale and cache sections")

@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()

if __name__ == "__main__":
    workflow.run_cli()
