#!/usr/bin/env python3
"""Task 002: Create detect.rs with ProfileFile parsing and walk-up search"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-002",
    version="v4",
    title="Create detect.rs with ProfileFile parsing and walk-up search",
    description="Create the new detect.rs module with two core functions: (1) ProfileFile struct + parsing from JSON, (2) walk-up directory search for .claude-profile.json using Path::ancestors(). This is the foundation of v4's profile detection system.",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="high",
    estimated_hours=2,
    phase="Phase 1: Foundation",
    dependencies=["task-001"],
    subtasks=[
        Subtask("task-002-1", "Define ProfileFile struct with Option<String> name and config_dir fields", 0.3),
        Subtask("task-002-2", "Implement ProfileFile parsing from JSON with validation", 0.5),
        Subtask("task-002-3", "Implement find_profile_file() walk-up search", 0.7),
        Subtask("task-002-4", "Add mod detect to main.rs", 0.2),
        Subtask("task-002-5", "Add unit tests for parsing and walk-up", 0.3),
    ],
    acceptance_criteria=[
        "ProfileFile struct has Option<String> fields for name and config_dir",
        '{"name": "work"} parses correctly with name=Some("work"), config_dir=None',
        '{"config_dir": "~/.claude-custom"} parses correctly',
        '{} or {"other": "field"} returns ProfileFileError',
        "Malformed JSON returns ConfigParseError",
        "find_profile_file() walks from cwd up to root looking for .claude-profile.json",
        "Returns None when no file found",
        "mod detect is declared in main.rs",
        "cargo build succeeds with no warnings",
    ],
    files=["src/detect.rs", "src/main.rs"],
    technical_notes="Use serde::Deserialize for ProfileFile. After deserializing, validate that at least one of name/config_dir is Some. Use std::env::current_dir() for starting point, Path::ancestors() for iteration. Return Option<(PathBuf, ProfileFile)> from find_profile_file. Use CyoloError::ProfileFileError for schema validation errors, CyoloError::ConfigParseError for JSON parse failures.",
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
    Read src/main.rs to see existing mod declarations.
    Read src/error.rs to confirm ProfileFileError and ConfigParseError variants.
    Read src/config.rs to understand CyoloConfig structure (needed for name resolution later).
    """
    pass


@workflow.step(2)
def create_detect_module():
    """
    Create src/detect.rs with:

    1. ProfileFile struct:
       - #[derive(Debug, Deserialize)]
       - name: Option<String>
       - config_dir: Option<String>

    2. ProfileFile::from_file(path: &Path) -> Result<ProfileFile, CyoloError>:
       - Read file, parse JSON via serde_json
       - On JSON error: return ConfigParseError
       - Validate at least one field is present, else ProfileFileError

    3. find_profile_file() -> Result<Option<(PathBuf, ProfileFile)>, CyoloError>:
       - let cwd = std::env::current_dir() (handle error as warning, return Ok(None))
       - for ancestor in cwd.ancestors():
           let candidate = ancestor.join(".claude-profile.json")
           if candidate.exists():
               return Ok(Some((candidate, ProfileFile::from_file(&candidate)?)))
       - return Ok(None)

    4. Add `mod detect;` to main.rs
    """
    pass


@workflow.step(3)
def add_tests():
    """
    Add unit tests in detect.rs:
    - test_parse_name_only: {"name": "work"} -> name=Some, config_dir=None
    - test_parse_config_dir_only: {"config_dir": "~/.x"} -> name=None, config_dir=Some
    - test_parse_both: {"name": "x", "config_dir": "y"} -> both Some
    - test_parse_empty_object: {} -> ProfileFileError
    - test_parse_malformed_json: "not json" -> ConfigParseError
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
    """All tests pass."""
    workflow.run_command("cargo test 2>&1")


@workflow.post_job
def save_learnings():
    """Record learnings."""
    workflow.update_memory("learning", "detect-module-v4", "Created detect.rs with ProfileFile parsing and walk-up search using Path::ancestors()")


@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()
