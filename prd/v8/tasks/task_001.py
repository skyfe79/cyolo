#!/usr/bin/env python3
"""Task 001: Extend DietOptions and parse_diet_args() with v8 flags"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-001",
    version="v8",
    title="Extend DietOptions and parse_diet_args() with stale/cache/profile/all flags",
    description="""Add four new CLI flags to the diet subcommand:
- --stale-days <N>: Option<u32>, detect projects inactive for N days
- --cache: bool, clean cache directories
- --profile <name>: Option<String>, target specific profile
- --all: bool, iterate all registered profiles

Validation rules:
- --profile and --all are mutually exclusive (error if both)
- --stale-days requires a positive integer value (> 0)
- --stale-days without a value or with non-numeric value is an error
- All existing flags (--apply, --force) continue to work
- Updated usage string""",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="high",
    estimated_hours=3,
    phase="Phase 1: CLI Parsing",
    dependencies=[],
    subtasks=[
        Subtask("task-001-1", "Add stale_days, cache, profile, all fields to DietOptions struct", 1),
        Subtask("task-001-2", "Extend parse_diet_args() to accept new flags with validation", 1),
        Subtask("task-001-3", "Write unit tests for all new flag combinations and error cases", 1),
    ],
    acceptance_criteria=[
        "DietOptions has stale_days: Option<u32>, cache: bool, profile: Option<String>, all: bool",
        "--stale-days 90 parses to stale_days: Some(90)",
        "--cache parses to cache: true",
        "--profile work parses to profile: Some(\"work\")",
        "--all parses to all: true",
        "--profile work --all returns error (mutually exclusive)",
        "--stale-days (missing value) returns error",
        "--stale-days 0 returns error (must be > 0)",
        "--stale-days abc returns error (not a number)",
        "All existing flag combinations still work",
        "cargo test passes",
    ],
    files=["src/diet.rs"],
    technical_notes="""Modify DietOptions struct at line ~25 and parse_diet_args() at line ~701.
parse_diet_args currently iterates args with a simple match. For --stale-days and --profile
which consume the next arg, switch to index-based iteration (while loop with index) or
use an iterator with .next() to consume value args.
Update the usage string to include all new flags.""",
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
    Read src/diet.rs and understand current DietOptions and parse_diet_args():
    1. DietOptions struct (line ~25): currently has apply: bool, force: bool
    2. parse_diet_args() (line ~701): simple for-loop matching --apply and --force
    3. Note how unknown args trigger error with usage message
    """
    pass

@workflow.step(2)
def implement_diet_options():
    """
    Extend DietOptions struct with four new fields:
    1. stale_days: Option<u32> — None when not specified
    2. cache: bool — default false
    3. profile: Option<String> — None when not specified
    4. all: bool — default false
    """
    pass

@workflow.step(3)
def implement_parse_diet_args():
    """
    Rewrite parse_diet_args() to handle value-consuming flags:
    1. Switch from for-loop to index-based iteration (while i < args.len())
    2. --stale-days: consume next arg, parse as u32, validate > 0
    3. --cache: set cache = true
    4. --profile: consume next arg as String
    5. --all: set all = true
    6. After parsing: validate --profile and --all not both set
    7. Update usage string to: "Usage: cyolo diet [--apply] [--force] [--stale-days <N>] [--cache] [--profile <name>] [--all]"
    """
    pass

@workflow.step(4)
def add_tests():
    """
    Add unit tests for new flag parsing:
    1. test_parse_args_stale_days: --stale-days 90 → Some(90)
    2. test_parse_args_stale_days_missing_value: --stale-days (no next) → error
    3. test_parse_args_stale_days_zero: --stale-days 0 → error
    4. test_parse_args_stale_days_negative: --stale-days -1 → error (parsed as unknown flag)
    5. test_parse_args_stale_days_non_numeric: --stale-days abc → error
    6. test_parse_args_cache: --cache → cache: true
    7. test_parse_args_profile: --profile work → Some("work")
    8. test_parse_args_profile_missing_value: --profile (no next) → error
    9. test_parse_args_all: --all → all: true
    10. test_parse_args_profile_and_all_exclusive: --profile x --all → error
    11. test_parse_args_all_flags_combined: --apply --force --stale-days 30 --cache --all → all set
    12. Verify existing tests still pass unchanged
    """
    pass

@workflow.step(5)
def codex_review_and_apply():
    """
    Run Codex AI code review and apply feedback:
    1. Run workflow.codex_review() to get review results
    2. Analyze the review feedback (code quality, security, performance issues)
    3. Apply necessary fixes based on the review
    4. Re-run cargo build && cargo test to ensure fixes don't break anything
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
    workflow.update_memory("learning", "v8-task-001", "Extended parse_diet_args with value-consuming flags")

@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()

if __name__ == "__main__":
    workflow.run_cli()
