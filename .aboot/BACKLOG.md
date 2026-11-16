# BACKLOG

## Current

## Later / likely

Should we move BACKLOG.md items once all are satisfied

> Yes — when an item is promoted to `.aboot/BOARD.md` and completed, move it here (renamed "Satisfied") or delete it; keep `Current` for open, workable items only.

## Satisfied

Release 0.2.0 (from `REQUIREMENTS.md`, 2026-11-15) — completed 2026-11-15: bumped 0.1.1 → 0.2.0 (the `todo` → `board` CLI rename and template v2/v3 layout moves are breaking for 0.x users), the old init commit was undone and everything re-committed as a single init commit, and `jsmithdev/aboot` was created with `gh` and `main` pushed.

`aboot status` staleness marker (from `BACKLOG.md`, 2026-11-15) — completed 2026-11-15: `aboot status` now shows `vN (out of date — run aboot update)` in the header when the manifest version is older than the current template version (and `"stale"` in `--json`), so staleness is visible without running `doctor`.

`.aboot/` directory layout (from `REQUIREMENTS.md`, 2026-11-15) — completed 2026-11-15: the five cold docs now live in the hidden `.aboot/` directory, with `AGENTS.md` + `BOARD.md` kept at the root; template v2 migrates existing projects via `aboot add`/`update` (manifest-gated, content preserved); the README and AGENTS.md template document the new layout.

- Decision: confirmed — hidden `.aboot/` for the five cold docs; `AGENTS.md` + `BOARD.md` at the root.
- Templates: five cold docs moved under `examples/.aboot/`; `builtin()` paths, `AGENTS.md` template references, and managed markers updated (v2).
- CLI: `expected_files` and the integration-test file lists use the `.aboot/` paths; board paths unchanged.
- Migration: `TEMPLATE_VERSION` bumped to 2; `aboot add`/`update` move root docs into `.aboot/` for pre-migration projects (manifest-gated, content preserved, each move reported); `doctor` flags the pre-migration layout.
- Docs: README file table, process, session-mode, and lifecycle sections updated; this repo migrated (dogfood).

Template refresh lifecycle (from `REQUIREMENTS.md`, 2026-11-15) — completed 2026-11-15: re-running `aboot add .` is now a true no-op (workflow files and the manifest's `files` list are preserved) and points at `aboot update` when the installed template is stale; the README documents the add → update lifecycle.

- README: documented the add → update lifecycle in "Managed sections & `aboot update`".
- `aboot add`: prints a staleness note when all files are skipped and the manifest is older than the current template version.
- `aboot add`: preserves the existing manifest `files` list when nothing was written (no more `files: []` rewrite).

## Later / speculative

- `--from <git-url>` — remote templates
- `aboot new --copy-from <existing-project>` — clone the workflow config from a sibling project
- Interactive prompts only when a TTY is detected (never in piped/agent use)
- Per-stack CI scaffolding (e.g. a minimal GitHub Actions workflow)
- Multi-project workspace management
