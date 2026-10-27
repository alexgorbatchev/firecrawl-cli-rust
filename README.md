`firecrawl` retrieves page content, discovers website links, searches the web, and manages autonomous Firecrawl jobs for terminal users and automation.

# What It Does

- **Page content:** Requests Markdown, HTML, links, screenshots, and structured extraction, with browser actions and page filters.
- **Site discovery:** Maps URLs and optionally includes their titles and descriptions.
- **Search:** Retrieves web, news, and image results, optionally with scraped page content.
- **Jobs:** Starts, inspects, and cancels autonomous tasks, with explicit waiting and credit budgets.
- **Automation:** Returns compact JSON with `AGENT=1` and saves results atomically when requested.

# How It Works

- Configure a Firecrawl API endpoint and, when required, an API key.
- Choose `page scrape`, `site map`, `web search`, or an `agent` operation.
- The endpoint fetches or discovers the requested content; results go to stdout or the selected output file.
- Job creation returns an ID immediately. Add `--wait` to poll that same job until completion.

# How it Really Works

- Requests use the configured endpoint's `/v2` API. The server performs website access and may consume credits; `--max-credits` sets a budget for autonomous tasks.
- There is no local configuration file or cache. Page-cache options control the server's cache. Schema and browser-action files are read before remote work.
- A single page format returns its content. Multiple formats, `--json`, or `AGENT=1` return JSON. Agent mode stays compact even with `--pretty`.
- Results go to stdout; warnings, timing, recovery IDs and errors go to stderr. Output files replace their destination atomically, with private permissions on Unix; their parent directory must already exist.
- Client deadlines use durations such as `30s`. Endpoint timeouts use milliseconds. `--wait-timeout` bounds the complete polling stage after creation.
- Redirects and automatic retries are disabled. If creation or cancellation loses its response, the remote operation can still have succeeded. Inspect server activity or the known job ID before retrying. Interrupting a wait does not cancel the remote job.
- Exit 0 means success, exit 1 means validation, API, timeout, failed waiting task, or output failure; exit 130 means local cancellation. Inspecting a failed task still succeeds when its status is retrieved successfully.

# Installation

This Rust implementation has no published binary release. The [source repository](https://github.com/alexgorbatchev/firecrawl-cli) identifies the Go tool; its releases and skill distribution path do not establish availability of this Rust port.

# Setup

- Set `FIRECRAWL_API_URL` to a [Firecrawl Cloud](https://docs.firecrawl.dev/introduction) or [self-hosted](https://docs.firecrawl.dev/contributing/self-host) endpoint, such as `https://api.firecrawl.dev`. Pass `--api-url` to override it.
- Set `FIRECRAWL_API_KEY` when the endpoint requires authentication. Self-hosted and keyless-capable endpoints can omit it; the server enforces access limits.

# Quick Start

```sh
firecrawl page scrape https://example.com
firecrawl site map https://example.com
AGENT=1 firecrawl web search 'Rust HTTP clients' --scrape
AGENT=1 firecrawl agent create 'Extract product prices' --urls https://example.com --max-credits 10
firecrawl --version
firecrawl skill
```

Sample Output, from the local map fixture:

```text
https://example.com/a
```

Use the returned job ID with `firecrawl agent status <JOB_ID>` or `firecrawl agent cancel <JOB_ID>`. Read the complete [usage guide](skills/firecrawl/SKILL.md) for request details, recovery and generated help commands.

# Options & Flags

Global options apply to domain commands. Flags have no short alias unless listed. Optional booleans accept `--flag` for true or `--flag=false`.

| Flag | Short | Default | Description |
| :--- | :--- | :--- | :--- |
| `--api-url <URL>` | — | `FIRECRAWL_API_URL`, otherwise unset | API base URL, required for remote operations. |
| `--api-key <KEY>` | — | `FIRECRAWL_API_KEY`, otherwise unset | Authentication, when required by the server. |
| `--request-timeout <DURATION>` | — | `5m` | Positive client deadline per request. |
| `--json` | — | `false` | Return JSON in human mode. |
| `--help` | `-h` | `false` | Print current command help offline. |
| `--version` | — | `false` | Print the raw version at the root; exclusive of other actions. |

`firecrawl page scrape <URL>`

| Flag | Short | Default | Description |
| :--- | :--- | :--- | :--- |
| `--format <FORMAT>` | — | `markdown` | Repeat or separate formats with commas; see the page reference for accepted names. |
| `--html` | — | `false` | Select HTML; conflicts with `--format`. |
| `--only-main-content[=BOOL]` | — | `true` | Include only main page content. |
| `--include-tags <TAGS>`, `--exclude-tags <TAGS>` | — | unset | Repeated or comma-separated tag filters. |
| `--wait-for <MS>` | — | unset | Rendering delay in milliseconds. |
| `--timeout-ms <MS>` | — | unset | Server scrape timeout, 1000–300000 ms. |
| `--mobile[=BOOL]` | — | unset | Select mobile browser behavior. |
| `--skip-tls-verification[=BOOL]` | — | unset | Set target-page certificate checking. |
| `--remove-base64-images[=BOOL]`, `--block-ads[=BOOL]` | — | unset | Set image removal and ad blocking. |
| `--proxy <MODE>` | — | unset | `auto`, `basic`, or `stealth`. |
| `--max-age <SECONDS>` | — | unset | Maximum server cache age. |
| `--store-in-cache[=BOOL]`, `--lockdown[=BOOL]`, `--redact-pii[=BOOL]` | — | unset | Server caching, cached-only requests, and personal-data redaction. |
| `--location-country <CODE>`, `--location-languages <LANGUAGES>` | — | unset | Two-letter country and preferred languages. |
| `--json-prompt <TEXT>` | — | unset | Describe structured extraction. |
| `--schema <JSON>`, `--schema-file <PATH>` | — | unset | Inline or file JSON object; mutually exclusive. |
| `--actions <JSON>`, `--actions-file <PATH>` | — | unset | Inline or file browser-action array; mutually exclusive. |
| `--screenshot`, `--full-page-screenshot` | — | `false` | Include a normal or full-page screenshot. |
| `--output <PATH>` | — | unset | Atomically save the result instead of stdout. |
| `--pretty` | — | `false` | Indent JSON in human mode. |
| `--timing` | — | `false` | Print request duration on stderr. |

`firecrawl site map <URL>`

| Flag | Short | Default | Description |
| :--- | :--- | :--- | :--- |
| `--search <TEXT>` | — | unset | Filter discovered links. |
| `--sitemap <MODE>` | — | unset | `skip`, `include`, or `only`. |
| `--include-subdomains[=BOOL]`, `--ignore-query-parameters[=BOOL]` | — | unset | Include subdomains and ignore query strings when deduplicating. |
| `--limit <COUNT>` | — | `100` | Maximum links, 1–100000. |
| `--timeout-ms <MS>` | — | unset | Positive server timeout in milliseconds. |
| `--location-country <CODE>`, `--location-languages <LANGUAGES>` | — | unset | Country and preferred languages. |
| `--detailed` | — | `false` | Include titles and descriptions in human output. |
| `--output <PATH>` | — | unset | Atomically save the result. |
| `--pretty` | — | `false` | Indent human-mode JSON. |

`firecrawl web search <QUERY>`

| Flag | Short | Default | Description |
| :--- | :--- | :--- | :--- |
| `--limit <COUNT>` | — | `5` | Results per source, 1–100. |
| `--sources <SOURCES>` | — | server default | `web`, `images`, `news`; repeat or separate with commas. |
| `--categories <CATEGORIES>` | — | unset | `github`, `research`, `pdf`, `developer`; developer is a separate index and cannot be combined with other categories. |
| `--tbs <FILTER>` | — | unset | Time filter such as `qdr:w`. |
| `--location <TEXT>` | — | unset | Geographic search location. |
| `--country <CODE>` | — | `US` | Two-letter country. |
| `--timeout-ms <MS>` | — | `60000` | Positive server search timeout. |
| `--ignore-invalid-urls[=BOOL]` | — | unset | Filter invalid URLs. |
| `--scrape` | — | `false` | Extract result content. |
| `--scrape-formats <FORMATS>` | — | `markdown` when scraping | Repeated or comma-separated page formats; requires `--scrape`. |
| `--scrape-only-main-content[=BOOL]` | — | `true` | Main-content selection when scraping. |
| `--output <PATH>` | — | unset | Atomically save the result. |
| `--pretty` | — | `false` | Indent human-mode JSON. |

`firecrawl agent create <PROMPT>`

| Flag | Short | Default | Description |
| :--- | :--- | :--- | :--- |
| `--urls <URLS>` | — | unset | Repeated or comma-separated starting URLs. |
| `--schema <JSON>`, `--schema-file <PATH>` | — | unset | Inline or file JSON object; mutually exclusive. |
| `--max-credits <COUNT>` | — | unset | Positive task credit budget. |
| `--strict-constrain-to-urls[=BOOL]` | — | unset | Constrain exploration; requires `--urls`. |
| `--model <MODEL>` | — | unset | `spark-2`, `spark-1-pro`, `spark-1-mini`. |
| `--webhook <URL_OR_JSON>` | — | unset | Notification URL or native webhook object. |
| `--wait` | — | `false` | Poll the created task. |
| `--poll-interval <DURATION>` | — | `5s` | Positive interval between status requests. |
| `--wait-timeout <DURATION>` | — | `5m` | Positive total polling deadline. |
| `--output <PATH>` | — | unset | Atomically save the result. |
| `--pretty` | — | `false` | Indent human-mode JSON. |

`firecrawl agent status <JOB_ID>` and `firecrawl agent cancel <JOB_ID>` each accept:

| Flag | Short | Default | Description |
| :--- | :--- | :--- | :--- |
| `--output <PATH>` | — | unset | Atomically save the result. |
| `--pretty` | — | `false` | Indent human-mode JSON. |

`firecrawl completion <SHELL>` generates Bash, Zsh, Fish, PowerShell or Elvish completions offline. `firecrawl skill` prints the Skills CLI install command and source skill URL offline. The Rust skill's publication is pending.

# License

[MIT](LICENSE), Alex Gorbatchev; upstream SDK attribution and licensing declarations are retained in [DEPENDENCIES.md](DEPENDENCIES.md).
