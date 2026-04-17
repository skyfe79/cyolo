#!/usr/bin/env python3
"""Task 002: Implement StaleProject and detect_stale_projects()"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-002",
    version="v8",
    title="Implement StaleProject struct and detect_stale_projects()",
    description="""Add stale project detection: walk session directories to find projects
with no activity in N or more days.

StaleProject struct holds path, last_activity_secs, history_size, session_size.

detect_stale_projects() iterates active projects in ~/.claude.json, computes session dir
name via project_path_to_session_dir(), walks files to find newest mtime, and flags
projects whose newest file is older than stale_days * 86400 seconds.

Edge cases:
- Project path does not exist on disk (orphan) → skip
- Session dir does not exist → not stale (benefit of the doubt)
- Empty session dir → not stale
- fs::Metadata::modified() returns Err → skip that file, warn to stderr""",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="high",
    estimated_hours=4,
    phase="Phase 2: Stale Detection",
    dependencies=["task-001"],
    subtasks=[
        Subtask("task-002-1", "Define StaleProject struct", 0.5),
        Subtask("task-002-2", "Implement detect_stale_projects() with mtime walk", 2),
        Subtask("task-002-3", "Write unit tests with tempdir mock session dirs", 1.5),
    ],
    acceptance_criteria=[
        "StaleProject struct has path, last_activity_secs, history_size, session_size fields",
        "Project with session files older than N days is detected as stale",
        "Project with session files newer than N days is not stale",
        "Project with no session directory is not stale",
        "Project with empty session directory is not stale",
        "Orphan projects (path doesn't exist) are skipped",
        "history_size reflects serialized size of the project's history array",
        "session_size reflects dir_size() of the session folder",
        "cargo test passes",
    ],
    files=["src/diet.rs"],
    technical_notes="""Use std::time::SystemTime::now() and Duration for age computation.
fs::Metadata::modified() returns Result<SystemTime> — handle Err by skipping the file.
Use SystemTime::duration_since(mtime) to get age; if mtime is in the future, skip.
Reuse existing project_path_to_session_dir() and dir_size() functions.
For history_size: look up the project key in parsed_json["projects"][path]["history"],
serialize it, and take len(). If no history array, history_size = 0.""",
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
    1. project_path_to_session_dir() — how project paths map to session dirs
    2. dir_size() — recursive size computation
    3. analyze_claude_json() — how parsed_json structure works
    4. OrphanedProject struct — pattern for similar struct design
    """
    pass

@workflow.step(2)
def implement_stale_project():
    """
    Add StaleProject struct after OrphanedSession:
    ```rust
    pub(crate) struct StaleProject {
        pub path: String,
        pub last_activity_secs: u64,
        pub history_size: u64,
        pub session_size: u64,
    }
    ```
    """
    pass

@workflow.step(3)
def implement_detect_stale():
    """
    Implement detect_stale_projects():
    1. Signature: (parsed_json: &Value, projects_dir: &Path, stale_days: u32) -> Vec<StaleProject>
    2. Extract projects object from parsed_json
    3. For each project path that exists on disk:
       a. Compute session dir name
       b. Check if session dir exists and is non-empty
       c. Walk all files recursively, find newest mtime
       d. Compare age to stale_days * 86400
       e. If stale: compute history_size from JSON, session_size via dir_size()
    4. Return Vec<StaleProject>
    """
    pass

@workflow.step(4)
def add_tests():
    """
    Write unit tests:
    1. test_detect_stale_old_files: create session dir with old-mtime files → detected
    2. test_detect_stale_recent_files: create session dir with recent files → not detected
    3. test_detect_stale_no_session_dir: project exists but no session dir → not stale
    4. test_detect_stale_empty_session_dir: session dir exists but empty → not stale
    5. test_detect_stale_orphan_skipped: project path doesn't exist → skipped
    6. test_detect_stale_history_size: verify history_size computed from JSON array

    Use filetime crate or manual mtime setting via std::fs::File::set_modified()
    to create files with specific ages. If set_modified is not available, use
    filetime::set_file_mtime or test with very large stale_days (0 days = all stale).
    """
    pass

@workflow.step(5)
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
    workflow.update_memory("learning", "v8-task-002", "Implemented stale project detection with mtime walk")

@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()

if __name__ == "__main__":
    workflow.run_cli()
