#!/usr/bin/env python3
"""Task 004: Coverage audit of detect.rs, symlink.rs, diet.rs"""

import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent))
from task_workflow import Task, Subtask, TaskWorkflow

task = Task(
    id="task-004",
    version="v10",
    title="Coverage audit — detect.rs, symlink.rs, diet.rs",
    description="""Audit the three high-coverage modules against product PRD requirements
and fill ONLY genuine gaps. Do not duplicate existing tests.

Module 1 — src/detect.rs (currently 15 tests): audit against product PRD §3.1
and §4.1–4.2. Required cases:
  - .claude-profile.json in cwd -> profile resolved to that name
  - .claude-profile.json found N levels up (walk-up works) -> profile resolved
  - No .claude-profile.json on walk-up path -> falls through to default
  - No .claude-profile.json AND no default set -> returns "no profile" (unset CLAUDE_CONFIG_DIR)
  - .claude-profile.json with {"name":"work"} where work is not in ~/.cyolo/config.json
    -> Err(CyoloError::ProfileNotFound { name: "work" })
  - .claude-profile.json with inline {"config_dir":"~/.claude-custom"} -> tilde expands, resolved inline
  - .claude-profile.json with malformed JSON -> Err(CyoloError::ProfileFileError { .. })
  - .claude-profile.json with both `name` and `config_dir` -> name wins (§4.1)

Module 2 — src/symlink.rs (currently 5 tests): audit against product PRD §4.6.
Required cases:
  - Source directory exists -> symlink created at target
  - Source directory does NOT exist -> auto-create source dir first, then symlink (§4.6 rule 1)
  - Source file does NOT exist -> skip symlink, log warning (§4.6 rule 2)
  - Target already exists -> skip symlink, log warning, no overwrite (§4.6 rule 3)
  - Registering ~/.claude itself as a profile -> NO symlinks created (§5.1 exception)
  - All tests use tempfile::TempDir and absolute paths
  - Gate with #[cfg(unix)]
  - Canonicalize both sides of any read_link comparison (macOS /tmp -> /private/tmp gotcha)

Module 3 — src/diet.rs (currently 117 tests): audit orphan-detection slice against
product PRD §6.1(A). Required cases:
  - Project path exists on filesystem -> kept, not orphan
  - Project path does NOT exist -> flagged orphan
  - Orphaned session folder ~/.claude/projects/<encoded-path>/ detected alongside entry
  - Reclaimable size = JSON bytes + session folder bytes

For each audit: if a required case is missing, ADD the test following that module's
existing style. If all cases are present, LEAVE the file untouched and document the
audit conclusion in the task's commit message body.""",
    status="completed",
    agent="meta-agent",
    skills=[],
    mcp_tools=[],
    priority="medium",
    estimated_hours=8,
    phase="Phase 1: Unit Tests",
    dependencies=[],
    subtasks=[
        Subtask("task-004-1", "Audit src/detect.rs against the 8 required cases; identify which are present and which are missing", 2),
        Subtask("task-004-2", "Audit src/symlink.rs against the 5 required cases; identify which are present and which are missing", 1.5),
        Subtask("task-004-3", "Audit src/diet.rs orphan-detection slice against the 4 required cases from §6.1(A); identify gaps in the existing 117 tests", 2),
        Subtask("task-004-4", "Add any missing tests following each module's existing style; gate symlink tests with #[cfg(unix)] and canonicalize() read_link comparisons", 1.5),
        Subtask("task-004-5", "Run cargo test --lib detect:: / symlink:: / diet:: and document audit conclusions (present vs added) in the commit body", 1),
    ],
    acceptance_criteria=[
        "Every required case enumerated in the description has a corresponding #[test] in the module (existing or added).",
        "cargo test --lib detect:: exits 0.",
        "cargo test --lib symlink:: exits 0.",
        "cargo test --lib diet:: exits 0.",
        "Total test count grep -rc \"#\\[test\\]\" src/ strictly increases vs v9 (or audit notes confirm zero gaps).",
        "No test writes outside a TempDir.",
        "All new symlink tests gated #[cfg(unix)] and canonicalize() both sides of read_link comparisons.",
    ],
    files=["src/detect.rs", "src/symlink.rs", "src/diet.rs"],
    technical_notes="""Audit workflow per module:
1. grep -n "#\\[test\\]" src/<module>.rs -> list current test names
2. For each required case, find the matching test (by behavior, not just name)
3. If not found, add a new #[test] in the existing #[cfg(test)] mod tests block

symlink tests must canonicalize:
    let left  = fs::read_link(&link_path).unwrap().canonicalize().unwrap();
    let right = source_dir.path().canonicalize().unwrap();
    assert_eq!(left, right);

Otherwise the assertion flakes on macOS where /tmp is itself a symlink to /private/tmp.

For diet.rs, the orphan-detection slice means tests that exercise the function deciding
whether a ~/.claude.json entry corresponds to an existing project path. The 117 tests
cover a lot — only add what's missing after a genuine read-through.

Commit message: feat(task-004): audit detect/symlink/diet test coverage and fill gaps

Commit body: document per-module conclusions:
  - detect.rs: <N> cases present, <M> added
  - symlink.rs: <N> cases present, <M> added
  - diet.rs: <N> cases present, <M> added""",
    web_search=[],
)

workflow = TaskWorkflow(task)

@workflow.pre_job
def load_context():
    """Load project context and skill files."""
    workflow.retrieve_memory("learning")
    workflow.read_skills()

@workflow.step(1)
def audit_detect():
    """
    Audit src/detect.rs (15 existing tests):
    1. Read src/detect.rs end-to-end
    2. For each of the 8 required cases in the description, find the matching test
    3. Build a gap list: (required_case, present?, test_name_or_None)
    4. Do NOT write any new test yet — just catalog
    """
    pass

@workflow.step(2)
def audit_symlink():
    """
    Audit src/symlink.rs (5 existing tests):
    1. Read src/symlink.rs end-to-end
    2. For each of the 5 required cases, find the matching test
    3. Check that existing tests already canonicalize() both sides of read_link
       comparisons (macOS /tmp gotcha) — flag any that don't
    4. Build a gap list
    """
    pass

@workflow.step(3)
def audit_diet():
    """
    Audit src/diet.rs orphan-detection slice (out of 117 existing tests):
    1. Search src/diet.rs for tests touching orphan/project-path logic
    2. For each of the 4 required cases from §6.1(A), find the matching test
    3. Build a gap list — DO NOT duplicate; 117 tests is thorough
    """
    pass

@workflow.step(4)
def fill_gaps():
    """
    Add only the genuinely missing tests identified in steps 1-3:
    1. Follow each module's existing test style (naming, setup helpers, assertions)
    2. detect.rs tests use tempfile::TempDir + canonicalize paths
    3. symlink.rs new tests: #[cfg(unix)] attribute + TempDir + canonicalize() on
       both sides of any read_link comparison
    4. diet.rs new tests: match the existing harness style, use TempDir
    5. If no gaps exist for a module, leave it untouched — do not pad
    """
    pass

@workflow.step(5)
def codex_review_and_apply():
    """
    Run Codex AI code review and apply feedback:
    1. Run workflow.codex_review() to get review results
    2. Analyze feedback (audit thoroughness, duplicate tests, missing canonicalize, wrong gating)
    3. Apply necessary fixes
    4. Re-run cargo build && cargo test to verify fixes
    """
    workflow.codex_review()

@workflow.verify
def check_builds():
    """Project builds without errors."""
    workflow.run_command("cargo build")

@workflow.verify
def check_detect_tests():
    """detect.rs module tests pass."""
    workflow.run_command("cargo test --lib detect::")

@workflow.verify
def check_symlink_tests():
    """symlink.rs module tests pass."""
    workflow.run_command("cargo test --lib symlink::")

@workflow.verify
def check_diet_tests():
    """diet.rs module tests pass."""
    workflow.run_command("cargo test --lib diet::")

@workflow.verify
def check_tests_pass():
    """Full test suite passes."""
    workflow.run_command("cargo test")

@workflow.post_job
def save_learnings():
    """Record learnings."""
    workflow.update_memory(
        "learning",
        "v10-task-004",
        "Audited detect/symlink/diet against product PRD required cases; filled only genuine gaps and documented audit conclusions per module",
    )

@workflow.post_job
def commit_changes():
    """Commit changes.

    Commit message: feat(task-004): audit detect/symlink/diet test coverage and fill gaps
    Commit body must document per-module conclusions (cases present vs added).
    """
    workflow.require_review_log()
    workflow.git_commit()

if __name__ == "__main__":
    workflow.run_cli()
