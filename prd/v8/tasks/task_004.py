#!/usr/bin/env python3
"""Task 004: Implement resolve_target_profiles()"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-004",
    version="v8",
    title="Implement resolve_target_profiles() for multi-profile diet targeting",
    description="""Determine which profile(s) to target based on DietOptions flags.

Three cases:
1. --profile <name>: load CyoloConfig, look up named profile, return its config_dir.
   Error if profile not found.
2. --all: load CyoloConfig, return all registered profiles' config_dir values.
   Error if no profiles registered.
3. Neither: use existing resolve_claude_home() logic (current profile detection),
   return single entry.

Returns Vec<(String, PathBuf)> — list of (profile_display_name, claude_home_path) tuples.
For the default case (no flags), use tilde-contracted path as display name.""",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="high",
    estimated_hours=3,
    phase="Phase 2: Profile Resolution",
    dependencies=["task-001"],
    subtasks=[
        Subtask("task-004-1", "Implement resolve_target_profiles() function", 1.5),
        Subtask("task-004-2", "Write unit tests for all three cases + error paths", 1.5),
    ],
    acceptance_criteria=[
        "--profile work resolves to the work profile's config_dir",
        "--profile nonexistent returns error with helpful message",
        "--all returns all registered profiles",
        "--all with no registered profiles returns error",
        "Default (no flags) returns single entry from resolve_claude_home()",
        "cargo test passes",
    ],
    files=["src/diet.rs", "src/config.rs"],
    technical_notes="""Use CyoloConfig::load() from config.rs to access registered profiles.
CyoloConfig has profiles: BTreeMap<String, Profile> and Profile has config_dir: PathBuf.

For the default case, reuse existing resolve_claude_home() which returns (home, claude_home).
Use the tilde_path() helper for display name.

For --profile: CyoloConfig::load() may fail if ~/.cyolo/config.json doesn't exist yet.
Handle gracefully with a clear error message suggesting 'cyolo profile add'.

For --all: iterate config.profiles.values(), map to (name, config_dir) tuples.

Note: config_dir in Profile may use ~ notation — need to expand it. Check if
CyoloConfig::load() already expands tildes or if we need to do it here.""",
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
    Read src/config.rs to understand:
    1. CyoloConfig::load() — how it reads ~/.cyolo/config.json
    2. Profile struct — what fields are available
    3. Whether config_dir paths are stored expanded or with ~
    4. Error handling when config.json doesn't exist

    Read src/diet.rs:
    1. resolve_claude_home() — current logic for finding claude home
    2. tilde_path() — display helper for contracting paths
    """
    pass

@workflow.step(2)
def implement_resolve():
    """
    Implement resolve_target_profiles(options: &DietOptions) -> Result<Vec<(String, PathBuf)>, CyoloError>:

    1. If options.profile is Some(name):
       - Load CyoloConfig
       - Look up name in config.profiles
       - If not found: return CyoloError with message "profile '{name}' not registered. Run: cyolo profile add {name}"
       - Return vec![(name, profile.config_dir)]

    2. If options.all:
       - Load CyoloConfig
       - If config.profiles is empty: return error "no profiles registered. Run: cyolo profile add <name>"
       - Return all profiles as vec of (name, config_dir)

    3. Default (neither):
       - Call resolve_claude_home() to get (home, claude_home)
       - Display name = tilde_path of claude_home
       - Return vec![(display_name, claude_home)]
    """
    pass

@workflow.step(3)
def add_tests():
    """
    Write unit tests (may need to mock CyoloConfig or use integration approach):
    1. test_resolve_default_no_flags: returns single entry from resolve_claude_home
    2. test_resolve_profile_not_found: --profile nonexistent → error
    3. test_resolve_all_no_profiles: --all with empty config → error

    Note: Testing --profile and --all paths that load CyoloConfig may require
    either mocking the config file or setting up temp config. Consider testing
    the logic paths that don't depend on filesystem if mocking is complex.
    At minimum, ensure the function compiles and the default path works.
    """
    pass

@workflow.step(4)
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
    workflow.update_memory("learning", "v8-task-004", "Implemented multi-profile resolution for diet")

@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()

if __name__ == "__main__":
    workflow.run_cli()
