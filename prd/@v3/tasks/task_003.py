#!/usr/bin/env python3
"""Task 003: Implement create_shared_symlinks function"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-003",
    version="v3",
    title="Implement create_shared_symlinks function",
    description="Implement the core create_shared_symlinks(config_dir) function in symlink.rs that creates all 6 symlinks from ~/.claude to the profile directory, with proper edge-case handling.",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="high",
    estimated_hours=2,
    phase="Phase 2: Core Implementation",
    dependencies=["task-002"],
    subtasks=[
        Subtask("task-003-1", "Self-reference check: skip all if is_source_dir", 0.3),
        Subtask("task-003-2", "Source handling: skip files if missing, create empty dirs if missing", 0.5),
        Subtask("task-003-3", "Target handling: detect existing with symlink_metadata, warn and skip", 0.5),
        Subtask("task-003-4", "Symlink creation with std::os::unix::fs::symlink", 0.3),
        Subtask("task-003-5", "Error handling: warn-and-continue on per-item failure", 0.4),
    ],
    acceptance_criteria=[
        "create_shared_symlinks skips all when is_source_dir returns true, prints info note",
        "File items: skip symlink when source missing (info message), continue to next",
        "Directory items: create empty dir at source with 0o755 when missing, then symlink",
        "Uses fs::symlink_metadata() (NOT Path::exists()) to detect existing targets including broken symlinks",
        "Existing target: warn and skip (no overwrite)",
        "Symlink error on one item does not prevent remaining items from processing",
        "Uses std::os::unix::fs::symlink for creation (absolute paths)",
        "cargo build succeeds",
    ],
    files=["src/symlink.rs"],
    technical_notes="""Key implementation details:
- Use fs::symlink_metadata() over Path::exists() — catches broken symlinks
- Use std::os::unix::fs::symlink(source, target) — handles both files and dirs on Unix
- Absolute symlinks only (profile dirs live at fixed paths)
- Warn via eprintln! for non-fatal issues (missing source file, existing target)
- Only return Err for unexpected I/O failures that prevent the entire operation
- Create empty source dirs with fs::create_dir_all + permissions 0o755
- Use std::os::unix::fs::PermissionsExt for setting dir permissions""",
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
    Read src/symlink.rs to understand current types and is_source_dir.
    Read src/error.rs to confirm SymlinkError variant signature.
    """
    pass


@workflow.step(2)
def implement_create_shared_symlinks():
    """
    Implement pub fn create_shared_symlinks(config_dir: &Path) -> Result<(), CyoloError>:

    1. If is_source_dir(config_dir) -> print note to stderr, return Ok(())
    2. Resolve source_base = dirs::home_dir().join(".claude")
    3. For each item in SHARED_ITEMS:
       a. source = source_base.join(item.name)
       b. target = config_dir.join(item.name)
       c. If source does not exist:
          - ItemKind::Directory -> fs::create_dir_all(&source) with 0o755 perms, then continue to symlink
          - ItemKind::File -> eprintln! info skip message, continue to next item
       d. Check target with fs::symlink_metadata(&target):
          - If Ok(_) -> target exists (file, dir, or symlink), eprintln! warn, continue
       e. Create symlink: std::os::unix::fs::symlink(&source, &target)
          - On error -> eprintln! warn with details, continue (do NOT abort)
    4. Return Ok(())

    Use CyoloError::SymlinkError only if the entire operation should fail (not used here — warn-and-continue).
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
    workflow.update_memory("learning", "create-symlinks-v3", "Implemented create_shared_symlinks with warn-and-continue pattern, symlink_metadata for detection, absolute symlinks")


@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()
