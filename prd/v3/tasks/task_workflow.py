"""
Task Workflow DSL for Agent-Pro

This file defines the PSEUDOCODE DSL used by task files (task_001.py, task_002.py, etc.).
Task files use Python-like syntax as a structured format, but they are NOT executable Python.

Orchestrators (maestro-loop, maestro-codex-loop, maestro-execute) READ task files as TEXT
and interpret the structure. They do NOT run them with python3.

How orchestrators interpret task files:
  - Task() fields → task metadata (id, title, description, files, etc.)
  - Subtask() entries → individual work items within a task
  - @workflow.pre_job functions → commands to run before implementation
  - @workflow.step(n) docstrings → ordered implementation instructions
  - @workflow.verify functions → acceptance criteria checks
  - @workflow.post_job functions → commands to run after implementation

Status updates:
  - Orchestrators change status directly in the .py file using the Edit tool
  - Example: Edit status="pending" → status="in-progress"
  - No JSON files are generated — the .py file IS the source of truth

Workflow execution order (interpreted by orchestrator):
  pre_job → step(1) → step(2) → ... → step(n) [review] → verify → post_job
"""

from dataclasses import dataclass, field
from typing import Callable, Optional, List, Tuple


class TaskStatus:
    """Valid task status values."""
    PENDING = "pending"
    IN_PROGRESS = "in-progress"
    COMPLETED = "completed"
    IGNORED = "ignored"


@dataclass
class Subtask:
    """Atomic unit of work within a task."""
    id: str
    title: str
    estimated_hours: float = 0
    status: str = "pending"


@dataclass
class Task:
    """Task definition with all metadata.

    Fields are read as text by orchestrators — not executed as Python.
    """
    id: str
    version: str
    title: str
    description: str
    agent: str = ""
    skills: List[str] = field(default_factory=list)
    mcp_tools: List[str] = field(default_factory=list)
    priority: str = "medium"
    estimated_hours: float = 0
    phase: str = ""
    dependencies: List[str] = field(default_factory=list)
    subtasks: List[Subtask] = field(default_factory=list)
    acceptance_criteria: List[str] = field(default_factory=list)
    files: List[str] = field(default_factory=list)
    technical_notes: str = ""
    web_search: List[str] = field(default_factory=list)
    status: str = "pending"


class TaskWorkflow:
    """
    Pseudocode DSL for defining task workflows.

    Decorators define the workflow structure that orchestrators interpret:
        @workflow.pre_job     - Setup before implementation
        @workflow.step(n)     - Ordered implementation steps (docstrings = instructions)
        @workflow.verify      - Acceptance criteria verification
        @workflow.post_job    - Cleanup and recording after implementation

    Command helpers describe actions for the orchestrator to perform:
        workflow.retrieve_memory("category")     → /retrieve-memory category
        workflow.update_memory("cat", "key", "v") → /update-memory cat key v
        workflow.read_skills()                    → READ each file in task.skills[]
        workflow.run_command("bun run build")     → Execute command via Bash
        workflow.use_mcp("github", "action")      → Use MCP tool
        workflow.codex_review()                    → Codex AI code review (use when Codex is reviewer)
        workflow.claude_review()                   → Claude AI code review (use when Claude is reviewer)
        workflow.git_commit()                     → git add + git commit

    Example task file:

        task = Task(id="task-001", version="v1", title="Create parser", ...)
        workflow = TaskWorkflow(task)

        @workflow.pre_job
        def load_context():
            workflow.retrieve_memory("learning")
            workflow.read_skills()

        @workflow.step(1)
        def investigate_codebase():
            \"\"\"
            Explore the codebase to understand existing patterns:
            - Read related files listed in task.files
            - Identify conventions and patterns to follow
            \"\"\"
            pass

        @workflow.step(2)
        def implement_core():
            \"\"\"
            Implement the core functionality:
            1. Create parser.ts with Parser class
            2. Add parse() method with error handling
            \"\"\"
            pass


        @workflow.step(3)   # Always the LAST step
        def codex_review_and_apply():
            \"\"\"\"\"\"
            Run Codex AI code review and apply feedback:
            1. Run workflow.codex_review() to get review results
            2. Analyze the review feedback
            3. Apply necessary fixes
            4. Re-run build/test to verify fixes
            \"\"\"\"\"\"
            workflow.codex_review()
        @workflow.verify
        def check_builds():
            \"\"\"Project builds without errors.\"\"\"
            workflow.run_command("bun run build")

        @workflow.post_job
        def save_learnings():
            workflow.update_memory("learning", "parser", "Implemented parser module")

        @workflow.post_job
        def commit_changes():
            workflow.require_review_log()   # MUST pass before git_commit
            workflow.git_commit()
    """

    def __init__(self, task: Task):
        self.task = task

    # --- Decorators (structural markers for orchestrator) ---

    def pre_job(self, fn: Callable) -> Callable:
        """Decorator: marks a function as pre-job phase."""
        return fn

    def step(self, order: int):
        """Decorator factory: marks an ordered implementation step.
        The function's DOCSTRING contains the implementation instructions.
        The function body (usually 'pass') is a placeholder."""
        def decorator(fn: Callable) -> Callable:
            return fn
        return decorator

    def verify(self, fn: Callable) -> Callable:
        """Decorator: marks a function as verification (acceptance criteria) check."""
        return fn

    def post_job(self, fn: Callable) -> Callable:
        """Decorator: marks a function as post-job phase."""
        return fn

    # --- Command helpers (pseudocode for orchestrator to interpret) ---

    def retrieve_memory(self, category: str, key: str = ""):
        """Orchestrator interprets as: /retrieve-memory {category} [{key}]"""
        pass

    def update_memory(self, category: str, key: str, value: str):
        """Orchestrator interprets as: /update-memory {category} {key} {value}"""
        pass

    def git_commit(self, message: str = ""):
        """Orchestrator interprets as: git add + git commit"""
        pass

    def run_command(self, cmd: str):
        """Orchestrator interprets as: execute {cmd} via Bash tool"""
        pass

    def read_skills(self):
        """Orchestrator interprets as: READ each file in task.skills[]"""
        pass

    def use_mcp(self, tool: str, action: str = ""):
        """Orchestrator interprets as: use MCP {tool} {action}"""
        pass

    def require_review_log(self):
        """Orchestrator interprets as: verify .review-discussion-{task.id}.md exists.
        MUST be called before git_commit() in post_job.
        If the file does not exist, orchestrator MUST block the commit
        and re-run the codex/claude review step."""
        pass

    def codex_review(self):
        """Orchestrator interprets as: Codex AI code review via app-server.
        Use when Codex is the reviewer (i.e., 'use claude' mode).
        When used in a @workflow.step, orchestrator must also analyze the review
        feedback and apply necessary fixes before proceeding."""
        pass

    def claude_review(self):
        """Orchestrator interprets as: Claude reviews changes via git diff.
        Use when Claude is the reviewer (i.e., 'use codex' mode).
        When used in a @workflow.step, orchestrator reads git diff, reviews code quality,
        provides feedback, and applies necessary fixes."""
        pass
