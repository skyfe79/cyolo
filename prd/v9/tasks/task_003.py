#!/usr/bin/env python3
"""Task 003: Color warnings and errors in profile.rs"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-003",
    version="v9",
    title="Color warnings and errors in profile.rs",
    description="""Add colored output to all user-facing messages in profile.rs using
owo-colors. Apply consistent color conventions:
- "error:" prefix → red bold
- "Usage:" prefix → yellow bold
- Success key-value output → values in green
- Profile list → active marker (*) in green bold, profile names in bold
- Suggestion/hint text → dimmed""",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="medium",
    estimated_hours=4,
    phase="Phase 3: Module Coloring",
    dependencies=["task-001"],
    subtasks=[
        Subtask("task-003-1", "Import OwoColorize in profile.rs", 0.5),
        Subtask("task-003-2", "Color error messages with red bold prefix", 1),
        Subtask("task-003-3", "Color usage messages with yellow bold prefix", 0.5),
        Subtask("task-003-4", "Color success output values in green", 1),
        Subtask("task-003-5", "Color profile list with bold names and green active marker", 0.5),
        Subtask("task-003-6", "Color suggestion/hint text as dimmed", 0.5),
    ],
    acceptance_criteria=[
        "profile.rs imports owo_colors::OwoColorize",
        "All eprintln! error messages use 'error:'.red().bold() prefix",
        "Usage messages use 'Usage:'.yellow().bold() prefix",
        "Success output values (paths, profile names) use .green()",
        "Profile list active marker (*) uses .green().bold()",
        "Profile names in list use .bold()",
        "Suggestion/hint text uses .dimmed()",
        "No raw uncolored error/warning/usage messages remain",
        "cargo build succeeds",
        "cargo test passes",
    ],
    files=["src/profile.rs"],
    technical_notes="""Import at the top of profile.rs:
use owo_colors::OwoColorize;

Pattern for error messages:
  eprintln!("{} message details", "error:".red().bold());

Pattern for usage:
  eprintln!("{} cyolo profile <command>", "Usage:".yellow().bold());

Pattern for success values:
  println!("Profile: {}", name.green());

Pattern for profile list:
  println!("{} {}", "*".green().bold(), name.bold());  // active profile
  println!("  {}", name.bold());                        // inactive profile

Pattern for suggestions:
  eprintln!("{}", "Try 'cyolo profile list' to see profiles.".dimmed());

Scan every eprintln!/println! in profile.rs and categorize each as
error, usage, success, list, or suggestion to apply the right coloring.""",
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
    Read src/profile.rs and catalog all user-facing output:
    1. Find all eprintln! calls — note line numbers and message types
    2. Find all println! calls — note line numbers and message types
    3. Categorize each as: error, usage/help, success, list display, or suggestion
    4. Note the current format strings to plan minimal edits
    """
    pass

@workflow.step(2)
def add_import_and_color_errors():
    """
    Add owo-colors import and color error messages:
    1. Add: use owo_colors::OwoColorize;
    2. For each eprintln! that shows an error, change prefix to "error:".red().bold()
    3. Ensure error message body remains readable (no color on the detail text)
    """
    pass

@workflow.step(3)
def color_usage_and_success():
    """
    Color usage and success messages:
    1. Usage messages: change prefix to "Usage:".yellow().bold()
    2. Success output: apply .green() to key values (paths, names)
    3. Keep labels/keys uncolored for readability
    """
    pass

@workflow.step(4)
def color_list_and_suggestions():
    """
    Color profile list display and suggestion text:
    1. Active profile marker (*): use "*".green().bold()
    2. Profile names in list: use name.bold()
    3. Suggestion/hint text: wrap in .dimmed()
    4. Review all remaining println!/eprintln! to ensure nothing is missed
    """
    pass

@workflow.step(5)
def codex_review_and_apply():
    """
    Run Codex AI code review and apply feedback:
    1. Run workflow.codex_review() to get review results
    2. Analyze the review feedback
    3. Apply necessary fixes
    4. Re-run cargo build && cargo test to verify fixes
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
    workflow.update_memory("learning", "v9-task-003", "Applied consistent color scheme to all profile.rs user messages")

@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()

if __name__ == "__main__":
    workflow.run_cli()
