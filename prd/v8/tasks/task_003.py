#!/usr/bin/env python3
"""Task 003: Implement CacheDir, measure_cache_dirs(), and remove_cache_contents()"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-003",
    version="v8",
    title="Implement CacheDir struct, measure_cache_dirs(), and remove_cache_contents()",
    description="""Add cache directory measurement and cleanup capabilities.

CacheDir struct holds name, path, and size for each cache directory.

measure_cache_dirs() checks three directories under claude_home: statsig/,
shell-snapshots/, file-history/. Returns Vec<CacheDir> for those that exist.

remove_cache_contents() iterates each CacheDir, removes files and subdirectories
within it, but preserves the directory itself. Uses tolerant delete pattern:
individual failures warn to stderr but don't abort.""",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="high",
    estimated_hours=3,
    phase="Phase 2: Cache Cleanup",
    dependencies=["task-001"],
    subtasks=[
        Subtask("task-003-1", "Define CacheDir struct", 0.5),
        Subtask("task-003-2", "Implement measure_cache_dirs()", 1),
        Subtask("task-003-3", "Implement remove_cache_contents()", 1),
        Subtask("task-003-4", "Write unit tests", 0.5),
    ],
    acceptance_criteria=[
        "CacheDir struct has name: String, path: PathBuf, size: u64",
        "measure_cache_dirs returns entries only for directories that exist",
        "Sizes computed correctly via dir_size()",
        "remove_cache_contents removes files within each cache dir",
        "Cache directories themselves are preserved (not deleted)",
        "Individual file removal failures warn to stderr but don't abort",
        "Returns (removed_count: usize, bytes_freed: u64)",
        "cargo test passes",
    ],
    files=["src/diet.rs"],
    technical_notes="""Three cache directories to check: "statsig", "shell-snapshots", "file-history".
Use the existing dir_size() for measurement.
For remove_cache_contents: iterate fs::read_dir() entries, use fs::remove_dir_all for subdirs
and fs::remove_file for files. Wrap each removal in match/if-let for tolerant errors.
Same pattern as rotate_backups() — warn on failure, continue.""",
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
    1. dir_size() — reuse for cache measurement
    2. rotate_backups() — tolerant delete pattern to follow
    3. OrphanedSession / OrphanedProject — struct patterns
    """
    pass

@workflow.step(2)
def implement_cache_dir():
    """
    Add CacheDir struct:
    ```rust
    pub(crate) struct CacheDir {
        pub name: String,
        pub path: PathBuf,
        pub size: u64,
    }
    ```
    """
    pass

@workflow.step(3)
def implement_measure():
    """
    Implement measure_cache_dirs(claude_home: &Path) -> Vec<CacheDir>:
    1. Define cache dir names: ["statsig", "shell-snapshots", "file-history"]
    2. For each: join with claude_home, check exists + is_dir
    3. If exists: compute dir_size(), create CacheDir entry
    4. Return vec of existing cache dirs
    """
    pass

@workflow.step(4)
def implement_remove():
    """
    Implement remove_cache_contents(cache_dirs: &[CacheDir]) -> Result<(usize, u64), CyoloError>:
    1. For each CacheDir: read_dir entries
    2. For each entry: if dir → remove_dir_all, if file → remove_file
    3. On individual failure: eprintln warning, continue
    4. Track removed_count (number of cache dirs processed) and bytes_freed (sum of sizes)
    5. Return Ok((removed_count, bytes_freed))
    """
    pass

@workflow.step(5)
def add_tests():
    """
    Write unit tests:
    1. test_measure_cache_existing: create all 3 cache dirs with files → returns 3 entries
    2. test_measure_cache_partial: only statsig exists → returns 1 entry
    3. test_measure_cache_none: no cache dirs exist → returns empty vec
    4. test_measure_cache_sizes: verify sizes match dir_size()
    5. test_remove_cache_contents_basic: populate cache dirs, remove, verify dirs still exist but empty
    6. test_remove_cache_contents_empty: empty cache dirs → (0, 0) or similar no-op
    7. test_remove_cache_contents_with_subdirs: nested dirs inside cache → all removed
    """
    pass

@workflow.step(6)
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
    workflow.update_memory("learning", "v8-task-003", "Implemented cache measurement and cleanup with tolerant delete")

@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()

if __name__ == "__main__":
    workflow.run_cli()
