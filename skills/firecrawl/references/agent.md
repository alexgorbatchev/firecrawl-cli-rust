---
created_on: 2026-10-14 17:18
last_modified: 2026-10-14 17:38
status: current
---

## `firecrawl agent create <PROMPT>`

Start one autonomous task. By default, return its job ID immediately. `--wait` prints the ID on stderr and polls that same task. A completed task yields native status JSON in agent/JSON mode or its data in human mode. A failed/cancelled task, polling error, or deadline fails with a recovery ID. Stopping the local command leaves remote work running.

| Flag | Type | Default | Contract |
| :--- | :--- | :--- | :--- |
| `--urls` | repeated/comma-separated string HTTP URLs | unset | Starting pages. |
| `--schema` | inline JSON object value | unset | Mutually exclusive with `--schema-file`. |
| `--schema-file` | path | unset | Read JSON object schema before task creation. |
| `--max-credits` | positive uint32 integer | unset | Server task credit budget. |
| `--strict-constrain-to-urls` | optional bool | unset | Requires `--urls`; constrain exploration to those pages. |
| `--model` | string enum | unset | `spark-2`, `spark-1-pro`, `spark-1-mini`; native SDK marks the latter two deprecated server aliases. |
| `--webhook` | HTTP URL or JSON object value | unset | Native webhook object with required `url` and optional `headers`, `metadata`, `events`; events are `started`, `action`, `completed`, `failed`, `cancelled`. |
| `--wait` | bool flag | `false` | Poll for completion. |
| `--poll-interval` | positive duration | `5s` | Delay between processing-state reads. |
| `--wait-timeout` | positive duration | `5m` | Total polling deadline, starting after confirmed creation. |
| `--output` | path | unset | Atomically save result. Waiting recovery ID still uses stderr. |
| `--pretty` | bool flag | `false` | Indent human-mode JSON. |

Optional bool accepts no value for true or `--strict-constrain-to-urls=false`. All listed options have no short aliases. Global options and `--help`/`-h` remain available. A response without a confirmed job ID fails; transport failure can leave an unconfirmed task, so inspect server activity before repeating creation.

## `firecrawl agent status <JOB_ID>`

Read current task status once. Human output includes status and native response details; agent/JSON output returns native status JSON including data, errors and credits where present. A successfully read failed task still exits 0. API/transport failures exit 1.

Accepts `--output PATH` (unset) and `--pretty` (false), plus global flags and `--help`/`-h`; no command-specific short aliases.

## `firecrawl agent cancel <JOB_ID>`

Send one native DELETE cancellation request. A confirmed cancellation prints `Cancelled job JOB_ID` in human mode or `{success:true,id:JOB_ID,cancelled:true}` in JSON/agent mode. An unconfirmed/failed response fails; inspect status before retrying an uncertain cancellation.

Accepts `--output PATH` (unset) and `--pretty` (false), plus global flags and `--help`/`-h`; no command-specific short aliases.
