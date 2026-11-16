# {{name}} — Agent Instructions

<!-- aboot:begin file="AGENTS.md" version="3" -->
## Workflow
- Session mode follows the workflow files provided as context (an explicit user instruction always overrides):
  - `.aboot/REQUIREMENTS.md` and/or `.aboot/BACKLOG.md`, no `.aboot/BOARD.md` → planning: refine the planning docs; do not modify `.aboot/BOARD.md` or run `aboot board add`/`done`.
  - `.aboot/BOARD.md` only, or no workflow files → execution: work the board per the rules below.
  - Both kinds → ask which mode the session is in before touching `.aboot/BOARD.md`.
- Workflow docs live in `.aboot/` (`REQUIREMENTS.md`, `BACKLOG.md`, `BOARD.md`, `DECISIONS.md`, `PROJECT.md`, `SESSIONS.md`); this file is the entry point.
- Maintain `.aboot/BOARD.md` as a checkbox list of remaining tasks.
- After completing each item, check it off in `.aboot/BOARD.md`.
- Work through items in order until all are checked.
- Do not stop until every item in `.aboot/BOARD.md` is `[x]`.
- When making a non-obvious design decision, append a one-line note to `.aboot/DECISIONS.md` explaining what and why.

## Project Facts
- Name: {{name}}
- Description: {{description}}
- Stack: {{stack}}
- Scaffolded: {{date}}

## Commands
{{stack_section}}

## Definition of Done
A task is done when all of the following hold:
- The change builds and all tests pass.
- Lint is clean.
- `.aboot/BOARD.md` is updated; any non-obvious decision is recorded in `.aboot/DECISIONS.md`.

## Rules
- Verify after every change: run the build/test commands above before starting the next task.
- Keep changes minimal and consistent with the existing codebase.
- Do not commit or push unless asked.
<!-- aboot:end -->
