# aboot

## What's this aboot?

Short for Agent-Bootstrap, `aboot` is a CLI tool (Rust) that scaffolds projects to quickly give AI coding agents an effective, repeatable agile workflow.

## Installation

Requires the [Rust toolchain](https://rustup.rs).

```sh
# from GitHub
cargo install aboot --git https://github.com/jsmithdev/aboot

# or from a local checkout
cargo install --path .
```

Both build a release binary and install it to `~/.cargo/bin` (on your `PATH` via rustup), so you can run `aboot` from any terminal.

## LLM Workflow

This project is set up so an LLM coding agent (or a human) can work on it, at any step, with minimal setup.

| File | Purpose |
| --- | --- |
| `AGENTS.md` | The agent's per-loop rules — entry point to the workflow | (Worker instructions) |
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

### Session mode

The planning docs in `.aboot/` are for planning, not per-loop agent work. The agent derives the session mode from which workflow files are provided as context (an explicit user instruction always overrides):

| Context | Mode | Agent behavior |
| --- | --- | --- |
| `.aboot/REQUIREMENTS.md` and/or `.aboot/BACKLOG.md`, no `.aboot/BOARD.md` | Planning | Refine the planning docs; never modify `.aboot/BOARD.md` or run `aboot board add`/`done` |
| `.aboot/BOARD.md` only, or no workflow files | Execution | Work `.aboot/BOARD.md` in order, checking items off, until every item is `[x]`; record decisions in `.aboot/DECISIONS.md` |
| Both kinds | Ambiguous | Ask which mode the session is in before touching `.aboot/BOARD.md` |

## Usage

### Commands

| Command | Purpose |
| --- | --- |
| `aboot new <name>` | Create a new project from the built-in template, `git init`, initial commit |
| `aboot add [path]` | Install the workflow files into an existing project (never overwrites by default) |
| `aboot status [path]` | Show which workflow files are present and `.aboot/BOARD.md` progress |
| `aboot board list\|add\|done` | Manage `.aboot/BOARD.md` items |
| `aboot doctor [path]` | Validate the workflow files (managed markers, manifest, version) |
| `aboot update [path]` | Re-apply the managed sections from the current template version |
| `aboot list [path]` | List aboot projects in a workspace directory |

### Global flags

- `--version` / `-v` — print the version and exit
- `--json` — machine-readable JSON output (errors are `{"error": "..."}`)
- `--quiet` / `-q` — suppress non-essential output

Exit codes: `0` success, `1` error, `2` usage error.

### `aboot new`

```sh
aboot new my-project --stack rust --description "A demo CLI" --task "Write first test"
```

| Flag | Effect |
| --- | --- |
| `--description <text>` | Seeded into `AGENTS.md`, `.aboot/PROJECT.md`, and the Python stub |
| `--task <text>` | First `.aboot/BOARD.md` item, placed under `NOW` |
| `--stack <rust\|typescript\|python\|go>` | Adds `.gitignore`, a build-config stub, and a stack section in `AGENTS.md` with build/test/lint commands |
| `--template <dir>` | Use a custom local template directory instead of the built-in one |
| `--no-git` | Skip `git init`/initial commit |
| `--git-remote <url>` | Add `origin` after the initial commit |
| `--license <SPDX>` | Write a `LICENSE` (short licenses embedded, others get a pointer file) |
| `--agent <agents\|claude\|both>` | Write `AGENTS.md` and/or `CLAUDE.md` (default `agents`) |
| `--dry-run` | Show what would be done without writing anything |
| `--force` | Allow an existing non-empty target directory and overwrite existing files |

### `aboot add`

Same flags as `new` minus `--no-git`/`--git-remote`/`--license`. `--stack` defaults to detection from build files (`Cargo.toml` → rust, `package.json` → typescript, `pyproject.toml`/`setup.py` → python, `go.mod` → go). Existing files are skipped unless `--force`; `README.md` is never touched. `--task` only applies when `.aboot/BOARD.md` is being created. `--md-only` (`-m`) installs only the markdown workflow files (no justfile, `.editorconfig`, or stack files); `--agent-only` (`-a`) installs only the agent instruction file; the two are mutually exclusive.

### Template variables

`{{name}}`, `{{description}}`, `{{date}}`, `{{stack}}` are rendered in every template file. An empty description renders as `—`.

### Managed sections & `aboot update`

`AGENTS.md`, `CLAUDE.md`, `justfile`, and `.editorconfig` contain aboot-managed regions marked with versioned markers (native comment syntax per file). `aboot update` replaces only those regions — user content outside the markers is preserved — recreates missing managed files, and bumps the version stamp in `.aboot.json`. `--force` re-adopts managed files that lost their markers.

**Lifecycle:** `aboot add` installs the workflow files once — re-running it skips existing files and never refreshes them. After you upgrade aboot, run `aboot update [path]` in each project to re-apply the managed sections in place (your edits outside the markers are preserved). `aboot doctor` flags projects whose template version is older than the one you're running.

### Releasing

Release builds are produced by [cargo-dist](https://cargo-dist.org): `cargo dist generate` produces the release workflow, and `cargo dist build` / `cargo dist create-local-sighash` etc. drive the single static binary build (see `dist-workspace.toml`).
