#!/usr/bin/env python3
"""Task 002: Create symlink.rs with types, constants, and self-reference detection"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-002",
    version="v3",
    title="Create symlink.rs with types, constants, and self-reference detection",
    description="Create the symlink module with SharedItem struct, ItemKind enum, SHARED_ITEMS constant (6 items), and is_source_dir() function. Also register mod symlink in main.rs.",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="high",
    estimated_hours=1.5,
    phase="Phase 1: Foundation",
    dependencies=["task-001"],
    subtasks=[
        Subtask("task-002-1", "Define ItemKind enum (File, Directory)", 0.2),
        Subtask("task-002-2", "Define SharedItem struct with name and kind fields", 0.2),
        Subtask("task-002-3", "Define SHARED_ITEMS constant with 6 items", 0.3),
        Subtask("task-002-4", "Implement is_source_dir() with canonicalize + fallback", 0.5),
        Subtask("task-002-5", "Add mod symlink to main.rs", 0.1),
    ],
    acceptance_criteria=[
        "src/symlink.rs exists with ItemKind, SharedItem, and SHARED_ITEMS",
        "SHARED_ITEMS contains exactly 6 items: CLAUDE.md, settings.json, settings.local.json, commands, skills, agents",
        "File items: CLAUDE.md, settings.json, settings.local.json",
        "Directory items: commands, skills, agents",
        "is_source_dir returns true when config_dir resolves to ~/.claude",
        "is_source_dir uses Path::canonicalize with fallback to direct comparison",
        "mod symlink declared in main.rs",
        "cargo build succeeds",
    ],
    files=["src/symlink.rs", "src/main.rs"],
    technical_notes="""ItemKind enum: File, Directory.
SharedItem struct: { name: &'static str, kind: ItemKind }.
SHARED_ITEMS: &[SharedItem] = &[...] with 6 entries.

is_source_dir(config_dir: &Path) -> bool:
1. Resolve ~/.claude via dirs::home_dir() + .join(".claude")
2. Try Path::canonicalize() on both paths, compare
3. If canonicalize fails (path may not exist yet), fall back to direct path comparison
4. Return true if config_dir IS ~/.claude""",
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
    Read src/main.rs to see current module declarations.
    Read src/error.rs to confirm SymlinkError variant exists (from task-001).
    Check Cargo.toml for dirs crate dependency.
    """
    pass


@workflow.step(2)
def create_symlink_module():
    """
    Create src/symlink.rs with:

    1. ItemKind enum (File, Directory) — derive Clone, Copy
    2. SharedItem struct { name: &'static str, kind: ItemKind }
    3. SHARED_ITEMS constant:
       - SharedItem { name: "CLAUDE.md", kind: ItemKind::File }
       - SharedItem { name: "settings.json", kind: ItemKind::File }
       - SharedItem { name: "settings.local.json", kind: ItemKind::File }
       - SharedItem { name: "commands", kind: ItemKind::Directory }
       - SharedItem { name: "skills", kind: ItemKind::Directory }
       - SharedItem { name: "agents", kind: ItemKind::Directory }

    4. pub fn is_source_dir(config_dir: &Path) -> bool
       - Get home dir via dirs::home_dir()
       - Build source_dir = home.join(".claude")
       - Try canonicalize both paths and compare
       - Fallback: compare paths directly if canonicalize fails
       - Return true if they match (config_dir IS the source)
    """
    pass


@workflow.step(3)
def register_module():
    """
    Add `mod symlink;` to src/main.rs alongside the other module declarations.
    Place it alphabetically among existing mod statements.
    """
    pass


@workflow.step(4)
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
    workflow.update_memory("learning", "symlink-module-v3", "Created symlink.rs with SharedItem types, SHARED_ITEMS constant, and is_source_dir using canonicalize+fallback")


@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()
