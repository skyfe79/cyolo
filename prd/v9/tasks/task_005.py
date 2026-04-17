#!/usr/bin/env python3
"""Task 005: Color diet report output"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-005",
    version="v9",
    title="Color diet report output",
    description="""Add ANSI color codes to the diet report and dispatch messages in diet.rs:

build_report_string():
- Section headers → bold
- Size values → cyan
- "(removable)" annotations → green
- "(keep)" annotations → dimmed
- "(history clearable)" annotations → yellow
- Tree-drawing characters (├── └── │) → dimmed

dispatch() messages:
- Error messages → "error:" red bold prefix
- Warning messages → "warning:" yellow bold prefix
- Success messages → "success:" or checkmark in green
- Action count messages → count numbers in green""",
    status="pending",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="medium",
    estimated_hours=5,
    phase="Phase 3: Module Coloring",
    dependencies=["task-001"],
    subtasks=[
        Subtask("task-005-1", "Import OwoColorize in diet.rs", 0.5),
        Subtask("task-005-2", "Color build_report_string() headers and sizes", 1.5),
        Subtask("task-005-3", "Color report annotations (removable/keep/clearable)", 1),
        Subtask("task-005-4", "Color tree-drawing characters as dimmed", 0.5),
        Subtask("task-005-5", "Color dispatch() error/warning/success/action messages", 1.5),
    ],
    acceptance_criteria=[
        "diet.rs imports owo_colors::OwoColorize",
        "Report section headers render in bold",
        "Size values (bytes, KB, MB, GB) render in cyan",
        "\"(removable)\" text renders in green",
        "\"(keep)\" text renders in dimmed",
        "\"(history clearable)\" text renders in yellow",
        "Tree characters (├── └── │) render in dimmed",
        "dispatch() error messages use 'error:'.red().bold() prefix",
        "dispatch() warning messages use 'warning:'.yellow().bold() prefix",
        "dispatch() success/action messages highlight counts in green",
        "cargo build succeeds",
        "cargo test passes",
    ],
    files=["src/diet.rs"],
    technical_notes="""Import at the top of diet.rs:
use owo_colors::OwoColorize;

For build_report_string(), embed colors directly into the formatted strings:
- Headers: format!("{}", "Orphaned Projects".bold())
- Sizes: format!("{}", format_size(bytes).cyan())
- Annotations: "(removable)".green(), "(keep)".dimmed(), "(history clearable)".yellow()
- Tree chars: "├──".dimmed(), "└──".dimmed(), "│".dimmed()

For dispatch():
- Error: eprintln!("{} {}", "error:".red().bold(), msg);
- Warning: eprintln!("{} {}", "warning:".yellow().bold(), msg);
- Success: println!("{} Removed {} items", "✓".green(), count.to_string().green());

Note: build_report_string() returns a String. The ANSI codes must be embedded
in the returned String so they render when printed. This means using format!()
with .red(), .bold(), etc. — which produce strings with embedded escape codes.

Be careful with existing tests that compare report strings — those tests may
need to be updated or the comparison may need to strip ANSI codes.""",
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
    Read src/diet.rs and understand the report generation:
    1. Find build_report_string() — understand its structure and output format
    2. Find dispatch() — catalog all eprintln!/println! calls
    3. Identify all string literals that need coloring
    4. Note any existing tests that assert on report string content
    5. Check for format_size() or similar helper functions
    """
    pass

@workflow.step(2)
def color_report_headers_and_sizes():
    """
    Color build_report_string() structural elements:
    1. Add import: use owo_colors::OwoColorize;
    2. Section headers (e.g., "Orphaned Projects", "Stale Projects", "Cache") → .bold()
    3. Size values from format_size() or similar → .cyan()
    4. Ensure the formatted String contains embedded ANSI escape codes
    """
    pass

@workflow.step(3)
def color_report_annotations_and_tree():
    """
    Color build_report_string() annotations and tree characters:
    1. "(removable)" → .green()
    2. "(keep)" → .dimmed()
    3. "(history clearable)" → .yellow()
    4. Tree-drawing characters (├── └── │) → .dimmed()
    5. Verify the complete report string looks correct with all colors
    """
    pass

@workflow.step(4)
def color_dispatch_messages():
    """
    Color dispatch() user-facing messages:
    1. Error messages → eprintln!("{} ...", "error:".red().bold(), ...)
    2. Warning messages → eprintln!("{} ...", "warning:".yellow().bold(), ...)
    3. Success messages → use green prefix or checkmark
    4. Action count messages → highlight the count number with .green()
    5. Review every eprintln!/println! in dispatch() for consistent coloring
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
    workflow.update_memory("learning", "v9-task-005", "Colored diet report with ANSI codes in build_report_string and dispatch messages")

@workflow.post_job
def commit_changes():
    """Commit changes."""
    workflow.require_review_log()
    workflow.git_commit()

if __name__ == "__main__":
    workflow.run_cli()
