#!/usr/bin/env python3
"""Task 006: Implement dispatch() — argument parsing and pipeline orchestration"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-006",
    version="v6",
    title="Implement dispatch() for diet argument parsing and pipeline orchestration",
    description="""Create the dispatch() entry point for the diet command. Parse CLI arguments
    (no args = dry-run, --apply = execute), resolve claude home directory, and orchestrate
    the analyze -> report -> apply pipeline.""",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="medium",
    estimated_hours=2,
    phase="Phase 5: Integration",
    dependencies=["task-002", "task-003", "task-004", "task-005"],
    subtasks=[
        Subtask("task-006-1", "Implement parse_diet_args() for --apply flag detection", 0.5),
        Subtask("task-006-2", "Implement resolve_claude_home() using dirs::home_dir()", 0.5),
        Subtask("task-006-3", "Implement dispatch() orchestrating full pipeline", 1),
        Subtask("task-006-4", "Unit tests for argument parsing and error cases", 0.5),
    ],
    acceptance_criteria=[
        "dispatch(&[]) runs analysis + prints report (dry-run)",
        "dispatch(&['--apply']) runs analysis + report + apply",
        "dispatch(&['--unknown']) returns error with usage message",
        "Uses dirs::home_dir() to resolve ~/.claude",
        "Constructs correct paths: ~/.claude.json, ~/.claude/projects/",
        "cargo test passes",
    ],
    files=["src/diet.rs"],
    technical_notes="""dispatch() signature: pub fn dispatch(args: &[String]) -> Result<(), CyoloError>

    Pipeline:
    1. parse args -> DietOptions { apply: bool }
    2. resolve claude home: dirs::home_dir() / ".claude"
    3. claude_json_path = home_dir / ".claude.json"
    4. projects_dir = claude_home / "projects"
    5. Call analyze_claude_json(&claude_json_path) -> AnalysisResult
    6. Call scan_session_folders(&projects_dir, orphaned_paths) -> (sessions, total_size)
    7. Build DietReport from results
    8. Call print_report(&report, options.apply)
    9. If options.apply: call apply(&report, &mut parsed_json, &claude_json_path)

    For unknown flags, return a CyoloError. Could reuse NotImplemented or add a new variant.
    Simple approach: check if any arg starts with '--' and isn't '--apply'.""",
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
    Read src/diet.rs to understand all implemented functions.
    Read src/cli.rs to understand how dispatch functions are called (profile::dispatch pattern).
    Read src/profile.rs to see dispatch pattern for reference.
    """
    pass

@workflow.step(2)
def implement_dispatch():
    """
    1. Implement parse_diet_args(args: &[String]) -> Result<DietOptions, CyoloError>:
       - No args -> DietOptions { apply: false }
       - ["--apply"] -> DietOptions { apply: true }
       - Any other arg -> Error with usage: "Usage: cyolo diet [--apply]"

    2. Implement resolve_claude_home() -> Result<PathBuf, CyoloError>:
       - dirs::home_dir().ok_or(CyoloError::...)
       - Return home.join(".claude")

    3. Implement pub fn dispatch(args: &[String]) -> Result<(), CyoloError>:
       - let options = parse_diet_args(args)?;
       - let home = dirs::home_dir().ok_or(...)?;
       - let claude_json_path = home.join(".claude.json");
       - let claude_home = home.join(".claude");
       - let projects_dir = claude_home.join("projects");
       - let analysis = analyze_claude_json(&claude_json_path)?;
       - let orphaned_paths: Vec<String> = analysis.orphaned_projects.iter().map(|p| p.path.clone()).collect();
       - let (sessions, session_dir_size) = scan_session_folders(&projects_dir, &orphaned_paths);
       - Build DietReport
       - print_report(&report, options.apply);
       - if options.apply { apply(&report, &mut analysis.parsed_json, &claude_json_path)?; }
       - Ok(())
    """
    pass

@workflow.step(3)
def add_tests():
    """
    Add unit tests:
    - test_parse_args_empty: no args -> DietOptions { apply: false }
    - test_parse_args_apply: ["--apply"] -> DietOptions { apply: true }
    - test_parse_args_unknown: ["--force"] -> Error
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
    workflow.update_memory("learning", "diet-dispatch", "Implemented dispatch with arg parsing and full pipeline orchestration")

@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()

if __name__ == "__main__":
    workflow.run_cli()
