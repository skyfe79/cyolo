#!/usr/bin/env python3
"""Task 003: Implement resolve_profile() with priority chain in detect.rs"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-003",
    version="v4",
    title="Implement resolve_profile() with priority chain in detect.rs",
    description="Add ResolvedProfile struct and resolve_profile() function to detect.rs. Implements the full priority chain: walk-up .claude-profile.json > default profile > None. Name variant resolves via CyoloConfig.profiles lookup. config_dir variant uses path directly with tilde expansion.",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="high",
    estimated_hours=2,
    phase="Phase 2: Core Logic",
    dependencies=["task-002"],
    subtasks=[
        Subtask("task-003-1", "Define ResolvedProfile struct with name, config_dir, source fields", 0.3),
        Subtask("task-003-2", "Implement resolve_profile() with walk-up, name lookup, and default fallback", 1.0),
        Subtask("task-003-3", "Expose or reuse expand_tilde() from profile.rs", 0.3),
        Subtask("task-003-4", "Add unit tests for resolution scenarios", 0.4),
    ],
    acceptance_criteria=[
        "ResolvedProfile has name (Option<String>), config_dir (PathBuf), and source (String) fields",
        "Walk-up file with name variant resolves via CyoloConfig.profiles lookup",
        "Walk-up file with unregistered name returns ProfileNotFound error",
        "Walk-up file with config_dir variant expands tilde and uses path directly",
        "No walk-up file + default set returns default profile",
        "No walk-up file + no default returns None",
        "resolve_profile() is pub for use by cli.rs and profile.rs",
        "cargo build succeeds with no warnings",
    ],
    files=["src/detect.rs", "src/profile.rs", "src/config.rs"],
    technical_notes="expand_tilde() currently lives in profile.rs. Either make it pub(crate) or move to a shared location. resolve_profile() needs to load CyoloConfig to look up profile names and check the default. Use CyoloConfig::load() which already exists in config.rs. The source field should be the path to .claude-profile.json (for walk-up) or 'default' (for default fallback).",
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
    Read src/detect.rs (from task-002) to see ProfileFile and find_profile_file().
    Read src/profile.rs to find expand_tilde() implementation.
    Read src/config.rs to understand CyoloConfig struct and profiles HashMap.
    """
    pass


@workflow.step(2)
def implement_resolve_profile():
    """
    Add to src/detect.rs:

    1. ResolvedProfile struct:
       - name: Option<String>
       - config_dir: PathBuf
       - source: String

    2. pub fn resolve_profile() -> Result<Option<ResolvedProfile>, CyoloError>:
       a. Call find_profile_file()
       b. If Some((path, profile_file)):
          - If profile_file.name is Some:
            - Load CyoloConfig, lookup name in profiles
            - Found: return ResolvedProfile { name, config_dir, source: path }
            - Not found: return Err(ProfileNotFound)
          - If profile_file.config_dir is Some:
            - expand_tilde on config_dir
            - return ResolvedProfile { name: None, config_dir, source: path }
       c. If None (no walk-up file):
          - Load CyoloConfig, check default field
          - If default set: resolve default name via profiles, return with source="default"
          - If no default: return Ok(None)

    3. Make expand_tilde() accessible:
       - In profile.rs, change fn expand_tilde to pub(crate) fn expand_tilde
       - Or re-export if needed
    """
    pass


@workflow.step(3)
def add_tests():
    """
    Add unit tests:
    - test_resolve_with_name: mock config with profile, verify resolution
    - test_resolve_with_config_dir: verify tilde expansion
    - test_resolve_default_fallback: no walk-up, default set
    - test_resolve_none: no walk-up, no default
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
    workflow.update_memory("learning", "resolve-profile-v4", "Implemented resolve_profile() with walk-up > default > None priority chain")


@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()
