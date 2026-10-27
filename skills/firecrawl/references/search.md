---
created_on: 2026-10-14 17:18
last_modified: 2026-10-14 17:38
status: current
---

## `firecrawl web search <QUERY>`

Search with one nonblank query of at most 500 Unicode characters. Human output shows Web/News/Image results, including snippets, image URLs, and requested scraped Markdown/HTML. Empty results print `No search results found.`. JSON/agent output returns the native `{success,data,warning}` response, plus `id`, `creditsUsed`, and `warnings` when returned by the server. Both warning forms also print on stderr.

| Flag | Type | Default | Contract |
| :--- | :--- | :--- | :--- |
| `--limit` | uint32 integer | `5` | 1–100 results per source. |
| `--sources` | repeated/comma-separated string enum | unset | `web`, `images`, `news`. |
| `--categories` | repeated/comma-separated string enum | unset | `github`, `research`, `pdf`, `developer`. Developer cannot be combined with other categories. |
| `--tbs` | string | unset | Time restriction, such as `qdr:d`, `qdr:w`. |
| `--location` | string | unset | Geographic search location. |
| `--country` | two-letter string | `US` | Country normalized to uppercase. |
| `--timeout-ms` | positive uint32 milliseconds | `60000` | Server search timeout. |
| `--ignore-invalid-urls` | optional bool | unset | Filter invalid URLs. |
| `--scrape` | bool flag | `false` | Read content from search results. |
| `--scrape-formats` | repeated/comma-separated string values | `markdown` when scraping | Requires `--scrape`; accepts the page scrape format names. |
| `--scrape-only-main-content` | optional bool | `true` | Applied when `--scrape` is selected. |
| `--output` | path | unset | Atomic result output file. |
| `--pretty` | bool flag | `false` | Indent human-mode JSON. |

Optional booleans accept no value for true or `--flag=false`. All listed options have no short aliases. Global options and `--help`/`-h` remain available. `github` restricts web search to GitHub pages. `developer` searches the separate repository/documentation index. The server documents a research-result schema change for 2026-11-16; this pinned SDK models current web/news/image results, so check that future contract before relying on research responses after that date.
