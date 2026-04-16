#!/usr/bin/env python3
"""Task 002: Create config module with schema, load, save, and atomic write"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-002",
    version="v2",
    title="Create config module with schema, load, save, and atomic write",
    description="Create src/config.rs implementing CyoloConfig and Profile structs, config_dir()/config_path() helpers, ensure_dir() with 0700 permissions, load() with missing-file fallback, and save() with atomic temp-file+fsync+rename pattern.",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="high",
    estimated_hours=3,
    phase="Phase 1: Foundation",
    dependencies=["task-001"],
    subtasks=[
        Subtask("task-002-1", "Define CyoloConfig and Profile structs with serde derives", 0.5),
        Subtask("task-002-2", "Implement config_dir() and config_path() using dirs::home_dir()", 0.5),
        Subtask("task-002-3", "Implement ensure_dir() with DirBuilder mode 0o700", 0.5),
        Subtask("task-002-4", "Implement load() with missing-file fallback to empty config", 0.5),
        Subtask("task-002-5", "Implement save() with atomic write (temp file + sync_all + rename)", 1.0),
    ],
    acceptance_criteria=[
        "CyoloConfig struct has default: Option<String> and profiles: BTreeMap<String, Profile>",
        "Profile struct has name: String and config_dir: PathBuf",
        "config_dir() returns ~/.cyolo/ PathBuf",
        "config_path() returns ~/.cyolo/config.json PathBuf",
        "ensure_dir() creates ~/.cyolo/ with 0700 permissions",
        "load() on missing file returns empty CyoloConfig (no error)",
        "load() on malformed JSON returns ConfigParseError",
        "save() produces pretty-printed JSON with sorted keys",
        "save() uses atomic write (temp file + sync_all + rename)",
        "mod config added to main.rs",
        "cargo build succeeds",
    ],
    files=["src/config.rs", "src/main.rs"],
    technical_notes="Use BTreeMap for deterministic key ordering. Use #[serde(default)] on optional fields. For atomic write: write to config.json.tmp in same dir, call file.sync_all(), then fs::rename(). Use std::os::unix::fs::DirBuilderExt for mode(). Return CyoloError::ConfigIoError for IO failures and CyoloError::ConfigParseError for JSON parse failures.",
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
    Read src/error.rs to see the error variants available (from task-001).
    Read src/main.rs to understand module structure.
    Read Cargo.toml to confirm serde, serde_json, dirs dependencies.
    """
    pass


@workflow.step(2)
def implement_config_module():
    """
    Create src/config.rs with the following:

    1. Structs:
       - CyoloConfig { default: Option<String>, profiles: BTreeMap<String, Profile> }
         with #[derive(Debug, Serialize, Deserialize, Default)]
         with #[serde(default)] on both fields
       - Profile { name: String, config_dir: PathBuf }
         with #[derive(Debug, Serialize, Deserialize, Clone)]

    2. Helper functions:
       - pub fn config_dir() -> Result<PathBuf, CyoloError>
         Returns ~/.cyolo/ using dirs::home_dir()
       - pub fn config_path() -> Result<PathBuf, CyoloError>
         Returns ~/.cyolo/config.json
       - pub fn ensure_dir() -> Result<(), CyoloError>
         Creates ~/.cyolo/ with mode 0o700 using DirBuilder

    3. CyoloConfig methods:
       - pub fn load() -> Result<Self, CyoloError>
         Read and parse config.json. Return Default if NotFound. ConfigParseError on bad JSON.
       - pub fn save(&self) -> Result<(), CyoloError>
         Atomic write: serialize pretty JSON, write to .tmp, sync_all(), rename().
    """
    pass


@workflow.step(3)
def register_module():
    """
    Add `mod config;` to src/main.rs (after existing mod declarations).
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
    workflow.update_memory("learning", "config-module-v2", "Created config.rs with atomic save and BTreeMap for sorted keys")


@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()
