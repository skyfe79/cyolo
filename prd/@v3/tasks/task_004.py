#!/usr/bin/env python3
"""Task 004: Integrate symlinks into profile add with --no-share flag"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-004",
    version="v3",
    title="Integrate symlinks into profile add with --no-share flag",
    description="Modify profile add to create symlinks after directory creation. Add --no-share flag to skip symlink creation. Handle flag in any position among the arguments.",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="high",
    estimated_hours=1.5,
    phase="Phase 3: Integration",
    dependencies=["task-003"],
    subtasks=[
        Subtask("task-004-1", "Parse --no-share flag from args (position-independent)", 0.5),
        Subtask("task-004-2", "Call create_shared_symlinks after dir creation unless --no-share", 0.5),
        Subtask("task-004-3", "Update confirmation message to mention symlink status", 0.3),
    ],
    acceptance_criteria=[
        "cyolo profile add work -> creates dir + symlinks + registers",
        "cyolo profile add work --no-share -> creates dir + registers (no symlinks)",
        "cyolo profile add --no-share work -> also works (flag position independent)",
        "cyolo profile add personal ~/.claude -> registers, skips symlinks (self-reference detected by create_shared_symlinks)",
        "Confirmation message mentions whether symlinks were created or skipped",
        "cargo build succeeds",
    ],
    files=["src/profile.rs"],
    technical_notes="""Execution order in profile add (after v3 changes):
1. Validate name not duplicate (existing)
2. Resolve config_dir (existing)
3. Create config_dir with 0700 (existing)
4. [NEW] Create symlinks unless --no-share or self-reference
5. Register profile in config (existing)
6. Save config (existing)

Flag parsing: filter --no-share from args before processing name and config-dir.
Both 'profile add work --no-share' and 'profile add --no-share work' must work.
Collect non-flag args separately from flag args.""",
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
    Read src/profile.rs to understand the current add() function implementation.
    Note argument parsing pattern, directory creation, and config save flow.
    """
    pass


@workflow.step(2)
def implement_no_share_and_symlinks():
    """
    Modify the add() function in src/profile.rs:

    1. Parse --no-share flag: iterate args, separate flags from positional args.
       - Collect positional args (name, optional config-dir) after filtering out --no-share
       - Set no_share = true if --no-share found anywhere in args

    2. After directory creation (existing step 3) and before config save (existing step 5), add:
       if !no_share {
           symlink::create_shared_symlinks(&config_dir)?;
       }

    3. Update the confirmation println! to indicate symlink status:
       - If no_share: "Added profile '<name>' at <dir> (no shared symlinks)"
       - If self-reference (is_source_dir): message from create_shared_symlinks handles it
       - Otherwise: "Added profile '<name>' at <dir> (shared symlinks created)"

    Add `use crate::symlink;` at the top of profile.rs.
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
    workflow.update_memory("learning", "profile-add-symlinks-v3", "Integrated symlink creation into profile add with position-independent --no-share flag")


@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()
