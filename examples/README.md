# {{name}}

## LLM Workflow

This project is set up so an LLM coding agent (or a human) can work on it with minimal setup.

| File | Purpose |
| --- | --- |
| `{{agent_file}}` | The agent's per-loop rules — entry point to the workflow | (Worker instructions) |
| `.aboot/BOARD.md` | The active dev board — what is in progress, next, and later | (DEV) |
| `.aboot/REQUIREMENTS.md` | Business needs gathered with the client — capabilities, personas, acceptance criteria | (client <--> BA) |
| `.aboot/BACKLOG.md` | Groomed, workable stories — sized, dependencies mapped, definition of done | (BA <--> Architect) |
| `.aboot/DECISIONS.md` | Log of non-obvious design decisions, one line each (what and why) | (Documentation) |
| `.aboot/PROJECT.md` | Goals, non-goals, and context for the agent | (Context) |
| `.aboot/SESSIONS.md` | Per-session handoff log — what was done, what's next | (Continuity) |

### Process

1. **Gather** — capture raw needs and ideas with the client in `.aboot/REQUIREMENTS.md`.
2. **Groom** — refine promising requirements into workable, sized stories in `.aboot/BACKLOG.md`.
3. **Plan** — pull the stories to work on into `.aboot/BOARD.md` as a checkbox list.
4. **Do** — the agent works through `.aboot/BOARD.md` in order, checking items off, and does not stop until every item is `[x]`.
5. **Record** — non-obvious design decisions get appended to `.aboot/DECISIONS.md`.

The planning docs in `.aboot/` are for planning, not per-loop agent work — during a work session the agent only manages `.aboot/BOARD.md` and `.aboot/DECISIONS.md`.
