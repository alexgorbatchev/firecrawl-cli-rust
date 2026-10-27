---
name: firecrawl
description: Use when scraping a web page, mapping a website, searching web results, or creating, inspecting, and cancelling Firecrawl agent jobs with the firecrawl CLI.
author: alexgorbatchev
metadata:
  created_on: 2026-10-14 17:18
  last_modified: 2026-10-14 17:38
  status: current
---

Set `AGENT=1` for compact JSON results. Read the relevant reference before executing domain commands. This skill describes the local Rust interface; its executable and skill are not published to the source repository's GitHub distribution path.

## Global interface

`firecrawl` displays the command tree. `firecrawl page`, `firecrawl site`, `firecrawl web`, and `firecrawl agent` require a verb. `firecrawl completion` requires a shell. `firecrawl help` prints root help; `firecrawl help help` describes the help interface.

All commands accept `--help`/`-h`, which prints help without API access. The root `--version` (bool, default `false`) prints only `0.1.0` followed by one newline and accepts no other action. Other flags have no short aliases.

| Global flag | Type | Default | Contract |
| :--- | :--- | :--- | :--- |
| `--api-url` | string: HTTP/HTTPS base URL | `FIRECRAWL_API_URL`, otherwise unset | Required for API commands. No embedded credentials, query or fragment. A base path is supported; the SDK appends `/v2`. |
| `--api-key` | string | `FIRECRAWL_API_KEY`, otherwise unset | Optional authentication for self-hosted/keyless-capable endpoints; the server enforces endpoint authorization. Hidden in help environment values. |
| `--request-timeout` | positive duration | `5m` | Per-request deadline, such as `30s` or `2m`. |
| `--json` | bool flag | `false` | Human mode also returns JSON. `AGENT=1`, `true`, or `yes` always selects compact JSON for domain results. |

Global flags can appear before or after domain commands. A schema is a JSON object. URLs are absolute HTTP/HTTPS URLs without embedded credentials. Job IDs contain 1–128 ASCII letters, digits, underscores or hyphens. Inline flags are parsed before operations; file input is read before API work.

## Results and side effects

Domain results go to stdout unless `--output PATH` selects atomic file replacement. Parent directories must exist. Output files use private temporary-file permissions (0600 on Unix). No local config/cache is maintained. Remote operations can consume credits or cause the server to fetch pages; agent creation starts remote work and cancellation mutates the selected task. Status polling reads the existing task.

`--pretty` defaults to false and indents JSON in human mode. Agent mode stays compact. Timing, warnings, and waiting job IDs use stderr. A raw single-format page result is text or JSON for its selected field; multiple formats or JSON mode return the native document. Map/search JSON includes the native response envelope. Job operations return native JSON except cancel's `{success,id,cancelled}` result.

Exit 0 means the command completed, exit 1 means validation, input, HTTP/API, timeout, failed waiting task, or output error; exit 130 means local cancellation. An agent status inspection reports a failed task's status successfully if the status API itself succeeds. A waiting create command fails for failed/cancelled terminal task states. Errors print once on stderr. Interrupted or failed creation/cancellation can have an uncertain remote outcome; inspect the task/server before retrying. Redirects and automatic request retries are disabled. Local cancellation stops polling/request work and does not automatically cancel a remote task.

## References

| Reference | Read for |
| :--- | :--- |
| [pages](references/pages.md) | `firecrawl page scrape`: formats, screenshots, browser actions and extraction schemas. |
| [map](references/map.md) | `firecrawl site map`: URL discovery and detailed link output. |
| [search](references/search.md) | `firecrawl web search`: sources, filters and scraped result content. |
| [agent](references/agent.md) | `firecrawl agent create`, `firecrawl agent status`, `firecrawl agent cancel`: credits, polling and recovery. |
| [help](references/help.md) | Every generated help query and shell completion command. |

## Workflows

```sh
AGENT=1 firecrawl --api-url https://api.firecrawl.dev page scrape https://example.com
AGENT=1 firecrawl site map https://example.com --limit 100
AGENT=1 firecrawl web search 'Rust HTTP clients' --sources web --scrape
AGENT=1 firecrawl agent create 'Extract product prices' --urls https://example.com --max-credits 10
AGENT=1 firecrawl agent status task-123
AGENT=1 firecrawl agent cancel task-123
```

The latter commands use the configured `FIRECRAWL_API_URL`; authenticated cloud operations use `FIRECRAWL_API_KEY`. Use the actual returned job ID in status/cancel requests.

## `firecrawl skill`

Prints the compiled Skills CLI installation command and source repository skill URL, identically in both modes, without API or filesystem access. It accepts no positional arguments or command-specific flags. The pointer's Rust skill publication is pending.
