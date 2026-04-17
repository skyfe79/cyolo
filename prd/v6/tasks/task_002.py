#!/usr/bin/env python3
"""Task 002: Implement analyze_claude_json() — orphaned project detection"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-002",
    version="v6",
    title="Implement analyze_claude_json() for orphaned project detection",
    description="""Parse ~/.claude.json using serde_json::Value and identify projects whose
    filesystem path no longer exists. Calculate approximate size contribution of each orphaned
    entry (serialized JSON length). Handle missing file, missing projects key, and empty projects
    object gracefully.""",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="high",
    estimated_hours=3,
    phase="Phase 2: Analysis",
    dependencies=["task-001"],
    subtasks=[
        Subtask("task-002-1", "Implement analyze_claude_json(claude_json_path) function", 1),
        Subtask("task-002-2", "Parse projects object, check Path::new(key).exists() for each", 1),
        Subtask("task-002-3", "Calculate entry_size via serde_json::to_string per orphaned entry", 1),
        Subtask("task-002-4", "Unit tests with tempdir + mock claude.json files", 1),
    ],
    acceptance_criteria=[
        "Correctly identifies project entries where the filesystem path no longer exists",
        "Handles missing ~/.claude.json gracefully (returns empty results, no error)",
        "Handles ~/.claude.json with no 'projects' key gracefully",
        "Handles empty 'projects' object gracefully",
        "Returns (Vec<OrphanedProject>, active_count, config_file_size, serde_json::Value) tuple",
        "cargo test passes with mock data",
    ],
    files=["src/diet.rs"],
    technical_notes="""Use serde_json::Value to preserve unknown fields. The function signature
    should return the parsed Value along with analysis results so that apply() can reuse it
    without re-parsing. Return type could be a struct or tuple:
    (Vec<OrphanedProject>, usize active_count, u64 config_size, Option<serde_json::Value>).
    Use std::fs::read_to_string + serde_json::from_str.
    For tests, use tempfile::TempDir to create mock claude.json files.""",
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
    Read src/diet.rs to understand the data structures from task-001.
    Read src/config.rs to see existing file read/parse patterns.
    """
    pass

@workflow.step(2)
def implement_analyze():
    """
    Implement analyze_claude_json(claude_json_path: &Path) -> Result<AnalysisResult, CyoloError>:

    1. If path doesn't exist, return Ok with empty results
    2. Read file contents via fs::read_to_string
    3. Parse as serde_json::Value
    4. Get "projects" key as object, if missing return empty results
    5. For each key in projects:
       - Check if Path::new(key).exists()
       - If not exists: create OrphanedProject with path and entry_size
       - entry_size = serde_json::to_string(&value).unwrap_or_default().len() as u64
    6. Count active projects (total - orphaned)
    7. Get config file size via fs::metadata().len()
    8. Return AnalysisResult with all data + the parsed Value for reuse

    Define AnalysisResult struct to hold:
    - orphaned_projects: Vec<OrphanedProject>
    - active_count: usize
    - config_file_size: u64
    - parsed_json: serde_json::Value (for reuse in apply)
    """
    pass

@workflow.step(3)
def add_tests():
    """
    Add unit tests for analyze_claude_json:
    - test_analyze_missing_file: path doesn't exist -> empty results, no error
    - test_analyze_no_projects_key: valid JSON without "projects" -> empty results
    - test_analyze_empty_projects: "projects": {} -> empty orphans, 0 active
    - test_analyze_detects_orphans: create temp dirs, add to mock JSON, delete some dirs -> detected
    - test_analyze_mixed: some paths exist, some don't -> correct orphan/active counts
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
    workflow.update_memory("learning", "diet-analyze", "Implemented analyze_claude_json with serde_json::Value for orphan detection")

@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()

if __name__ == "__main__":
    workflow.run_cli()
