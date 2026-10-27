---
created_on: 2026-10-14 17:18
last_modified: 2026-10-14 17:38
status: current
---

## `firecrawl site map <URL>`

Discover a site's links synchronously. Human output lists URLs, or `No URLs found.`; `--detailed` also includes titles and descriptions. JSON/agent output returns `{success,links,warning}` as modeled by the native SDK.

| Flag | Type | Default | Contract |
| :--- | :--- | :--- | :--- |
| `--search` | string | unset | Query to filter discovered links. |
| `--sitemap` | string enum | unset | `skip`, `include`, `only`. |
| `--include-subdomains` | optional bool | unset | Include subdomain links. |
| `--ignore-query-parameters` | optional bool | unset | Deduplicate without query strings. |
| `--limit` | uint32 integer | `100` | 1–100000 links; this default is explicitly sent. |
| `--timeout-ms` | positive uint32 milliseconds | unset | Server mapping limit in milliseconds. |
| `--location-country` | two-letter string | unset | Country normalized to uppercase. |
| `--location-languages` | repeated/comma-separated string values | unset | Preferred languages. |
| `--detailed` | bool flag | `false` | Include human-readable title/description details. |
| `--output` | path | unset | Atomic result output file. |
| `--pretty` | bool flag | `false` | Indent human-mode JSON. |

Optional booleans accept no value for true or `--flag=false`. All listed options have no short aliases. Global options and `--help`/`-h` remain available. Sitemap accepts modes rather than a custom sitemap URL.
