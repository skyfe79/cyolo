#!/usr/bin/env python3
"""Task 002: Implement profile_init() to create .claude-profile.json"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-002",
    version="v5",
    title="Implement profile_init() to create .claude-profile.json in cwd",
    description=(
        "Add profile_init(args) function to src/profile.rs that creates "
        ".claude-profile.json in the current working directory. Resolves profile name "
        "from argument or falls back to config.default. Validates name exists in registry. "
        "Refuses to overwrite existing file."
    ),
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="high",
    estimated_hours=2,
    phase="Phase 1: Core Functions",
    dependencies=["task-001"],
    subtasks=[
        Subtask("task-002-1", "Implement profile_init() function in profile.rs", 1.5),
        Subtask("task-002-2", "Handle edge cases: no name + no default, file exists", 0.5),
    ],
    acceptance_criteria=[
        "cyolo profile init work with registered 'work' creates .claude-profile.json with {\"name\": \"work\"}",
        "cyolo profile init unknown with unregistered name returns ProfileNotFound error",
        "cyolo profile init (no name) with default set uses the default profile name",
        "cyolo profile init (no name, no default) errors: 'No profile name given and no default profile set.'",
        "cyolo profile init work when .claude-profile.json already exists errors without overwriting",
        "cargo build succeeds",
        "cargo test passes",
    ],
    files=["src/profile.rs"],
    technical_notes=(
        "Use std::env::current_dir() for cwd. Use serde_json::to_string_pretty for JSON output. "
        "Always write the 'name' form: {\"name\": \"<name>\"}. Check Path::exists() before writing. "
        "For the no-name case, read config.default; if None, print usage error and return NonZeroExit(1). "
        "Depends on task-001 because the no-name path reads config.default."
    ),
    web_search=[],
)

workflow = TaskWorkflow(task)

@workflow.pre_job
def load_context():
    """Load project context and skill files."""
    workflow.retrieve_memory("learning")
    workflow.read_skills()

@workflow.step(1)
def investigate_codebase():
    """
    Read existing code:
    - src/profile.rs: see expand_tilde(), add(), current() for patterns
    - src/config.rs: CyoloConfig::load() for reading default
    - PRD F1 section for exact behavior specification
    """
    pass

@workflow.step(2)
def implement_profile_init():
    """
    Implement profile_init(args: &[String]) in src/profile.rs:

    1. Call config::ensure_dir()? and load config via CyoloConfig::load()?

    2. Resolve the profile name:
       - If args has one positional arg → use that as name
       - If args is empty → use config.default
         - If config.default is None → error:
           eprintln!("No profile name given and no default profile set.")
           eprintln!("Usage: cyolo profile init <name>")
           return Err(NonZeroExit(1))
       - If more than one arg → usage error

    3. Validate the resolved name exists in config.profiles
       - If not: return Err(ProfileNotFound { name })

    4. Check if .claude-profile.json already exists in cwd:
       - let cwd = std::env::current_dir().map_err(...)?;
       - let profile_path = cwd.join(".claude-profile.json");
       - if profile_path.exists() {
           eprintln!("cyolo: .claude-profile.json already exists in {}", cwd.display());
           return Err(NonZeroExit(1));
         }

    5. Write the file:
       - Create a simple JSON object: {"name": "<resolved_name>"}
       - Use serde_json::json!({"name": name}) and serde_json::to_string_pretty()
       - std::fs::write(&profile_path, contents + "\n")
       - println!("Created .claude-profile.json (profile: {name})")
    """
    pass

@workflow.step(3)
def codex_review_and_apply():
    """
    Run Codex AI code review and apply feedback:
    1. Run workflow.codex_review() to get review results
    2. Analyze the review feedback (code quality, security, performance issues)
    3. Apply necessary fixes based on the review
    4. Re-run cargo build and cargo test to ensure fixes don't break anything
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
    """Record learnings from implementation."""
    workflow.update_memory("learning", "profile-init", "Implemented profile init with name resolution and cwd write")

@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()

if __name__ == "__main__":
    workflow.run_cli()
