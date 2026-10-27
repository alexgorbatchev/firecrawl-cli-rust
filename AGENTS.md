---
created_on: 2026-10-14 17:18
last_modified: 2026-10-14 17:18
status: current
---

# Firecrawl CLI

`firecrawl` scrapes pages, maps websites, searches the web, and manages autonomous Firecrawl tasks.

## Commands

- Run: `just run page scrape https://example.com`.
- Agent mode: `just run-ai page scrape https://example.com`.
- Test: `just test`.
- Format and lint: `just lint`.
- Full gates: `just check`.
- Release build: `just build`, producing `bin/firecrawl`.

## Contracts

- Use the native official SDK options and methods. Maintain the documented small patches in `vendor/firecrawl`; preserve its attribution and licensing declarations. See `DEPENDENCIES.md`.
- Keep API timeouts in milliseconds distinct from client and polling durations. Validate local schema/actions before any remote operation.
- Keep redirects and automatic HTTP retries disabled. Creation or cancellation can succeed remotely when transport fails; retain uncertainty and job IDs rather than replaying writes.
- Waiting uses native start/status methods and an explicit deadline. A failed or cancelled task fails the waiting command; polling never creates another task.
- Timing, warnings, recovery IDs, and errors belong on stderr. JSON stdout must remain one valid document. Keep secret values out of help, diagnostics, and evidence.
- Replace output atomically with a private temporary file in its destination directory. Never create missing parent directories or alter local input files implicitly.
- Generate shell completions into memory before the fallible output write; the native Fish generator can panic when handed a failing writer directly.
- Maintain `skills/firecrawl/SKILL.md` and its references in the same change as any command, argument, option, default, environment, output, or side-effect change. Update metadata and run live interface tests. The skill is a single repository file, not embedded in the executable.

## Verification and boundaries

- Any runtime-result change requires a corresponding test-file change and at least 90% line coverage; `scripts/` is excluded. Record meaningful red/green and actual disabled/restored behavior under `.tmp/`.
- Use local HTTP servers and isolated files for tests. Never use live API credentials or perform paid tasks as a verification shortcut.
- Never publish releases, tags, packages, or production deployments without explicit user authorization.
- Scope formatting to this project's explicit files; sibling path dependencies have separate owners.
- Keep standing project rules here, updating existing rules in place. Keep task history and verification results outside agent instructions.
