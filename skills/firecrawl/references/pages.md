---
created_on: 2026-10-14 17:18
last_modified: 2026-10-14 17:38
status: current
---

## `firecrawl page scrape <URL>`

Extract content from one page. A default request asks for Markdown and only main content. A single format prints that format's content; a missing field falls back to the document JSON. Multiple formats, `--json`, or agent mode print document JSON. Structured-extraction options add JSON output when it is not already requested; screenshot flags similarly add screenshots.

| Flag | Type | Default | Contract |
| :--- | :--- | :--- | :--- |
| `--format` | repeated/comma-separated string values | `markdown` | `markdown`, `html`, `rawHtml`, `links`, `screenshot`, `json`, `images`, `summary`, `changeTracking`, `attributes`, `branding`. |
| `--html` | bool flag | `false` | Select HTML; mutually exclusive with `--format`. |
| `--only-main-content` | optional bool | `true` | Use `--only-main-content=false` to include other page content. |
| `--include-tags`, `--exclude-tags` | repeated/comma-separated string values | unset | HTML tag filters. |
| `--wait-for` | uint32 milliseconds | unset | Rendering wait after page load; zero is accepted. |
| `--timeout-ms` | uint32 milliseconds | unset | Server execution limit, 1000–300000. Client deadline is separately `--request-timeout`. |
| `--mobile` | optional bool | unset | Mobile browser selection. |
| `--skip-tls-verification` | optional bool | unset | Target page certificate-check policy. |
| `--remove-base64-images`, `--block-ads` | optional bool | unset | Embedded image removal and ad blocking. |
| `--proxy` | string enum | unset | `auto`, `basic`, `stealth`. |
| `--max-age` | uint32 seconds | unset | Maximum server cache age; zero is accepted. |
| `--store-in-cache`, `--lockdown`, `--redact-pii` | optional bool | unset | Server cache, cached-only fetch, personal-information redaction. |
| `--location-country` | two-letter string | unset | Country is normalized to uppercase. |
| `--location-languages` | repeated/comma-separated string values | unset | Preferred languages. |
| `--json-prompt` | string | unset | Extraction prompt in the JSON format object. |
| `--schema` | inline JSON object value | unset | Extraction schema; mutually exclusive with `--schema-file`. |
| `--schema-file` | path | unset | Read a JSON object schema before API work. |
| `--actions` | JSON array value | unset | Native browser actions; mutually exclusive with `--actions-file`. |
| `--actions-file` | path | unset | Read native browser actions from JSON before API work. |
| `--screenshot` | bool flag | `false` | Include a screenshot. |
| `--full-page-screenshot` | bool flag | `false` | Include a screenshot format object with `fullPage:true`. |
| `--output` | path | unset | Atomically save content instead of writing stdout. |
| `--pretty` | bool flag | `false` | Indent human-mode JSON. |
| `--timing` | bool flag | `false` | Print request duration on stderr. |

An optional bool without a value sets true; explicitly set false using `--flag=false`. Unset values defer to server defaults. All listed options have no short aliases. Global options and `--help`/`-h` remain available.

Actions use the SDK's discriminated `type` values: `wait`, `screenshot`, `click`, `write`, `press`, `scroll`, `scrape`, `executeJavascript`, `pdf`. Fields vary by action; for example `[{"type":"wait","milliseconds":1000}]`. An invalid native action fails before a request. The official [scrape API](https://docs.firecrawl.dev/api-reference/endpoint/scrape) defines action fields and server constraints.

```sh
AGENT=1 firecrawl page scrape https://example.com --format json --json-prompt 'Extract prices' --schema '{"type":"object"}'
firecrawl page scrape https://example.com --format screenshot --full-page-screenshot --output page.json
```
