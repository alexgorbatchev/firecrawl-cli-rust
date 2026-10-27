---
created_on: 2026-10-14 17:18
last_modified: 2026-10-14 17:38
status: current
---

# Dependencies and source contracts

The source CLI is [alexgorbatchev/firecrawl-cli](https://github.com/alexgorbatchev/firecrawl-cli), MIT, copyright 2026 Alex Gorbatchev. Its pinned Go SDK is `github.com/firecrawl/firecrawl/apps/go-sdk` v1.7.1.

| Go dependency or functionality | Rust replacement |
| :--- | :--- |
| Official Firecrawl Go SDK | Official `firecrawl` 2.21.5, consumed directly through native options and client methods. |
| Cobra/pflag | Clap 4.6.7 through `cli-command-kit`; native typed validation and completion generation. |
| HTTP client and operation context | Reqwest 0.12.28 and Tokio deadlines/cancellation. |
| JSON and file output | Serde JSON and same-directory `tempfile` atomic persistence. |
| Duration flags | `humantime` duration parsing. |

## Native SDK patches

`vendor/firecrawl` contains the official crates.io 2.21.5 source, Cargo metadata and README. Its package metadata and README declare MIT. Cargo attributes Gergő Móricz, sanix-darker and kkharji. The distributed crate and current upstream Rust SDK directory contain no standalone LICENSE file; retain the actual upstream declarations and attribution rather than substituting the repository root's AGPL license for the explicitly MIT-declared Rust package. The pinned Go SDK's standalone LICENSE is AGPL-3.0 despite this CLI's MIT license; it is not copied into the Rust implementation.

The maintained corrections are native API changes, not a request-translation facade:

- `src/client.rs`: consuming `with_http_client` supplies the native Reqwest client with explicit redirect/retry/timeout policies; a non-2xx response cannot become success merely because its JSON has a success-shaped schema.
- `src/types.rs`: native `Format::JsonWithOptions` and `Format::ScreenshotWithOptions` serialize per-format objects and deserialize those objects; their option types gain equality derives. Screenshot browser actions use the API's `fullPage` field. Native `SearchCategory::Developer` names the current documented repository/documentation index category.
- `src/agent.rs`: serialize `strict_constrain_to_urls` as `strictConstrainToURLs`.
- `src/search.rs`: serialize `ignore_invalid_urls` as `ignoreInvalidURLs`; retain native response `id`, `creditsUsed`, and plural `warnings` fields rather than discarding them.

Real local HTTP regressions validate the actual bytes received by the endpoint. API references: [scrape](https://docs.firecrawl.dev/api-reference/endpoint/scrape), [map](https://docs.firecrawl.dev/api-reference/endpoint/map), [search](https://docs.firecrawl.dev/api-reference/endpoint/search), [agent](https://docs.firecrawl.dev/api-reference/endpoint/agent).

## Behavior corrections

The subject-first interface replaces source verb commands and its direct-URL root shortcut with `page scrape`, `site map`, `web search`, and separate `agent create/status/cancel` operations. This is a Rust interface, with no legacy argument facade.

Source handlers ignore full-page screenshots, search country/timeout, and agent wait/poll/timeout flags. Rust applies these through native API fields and an explicit polling workflow. `agent create` starts once and returns the job ID by default; `--wait` enables polling. `--request-timeout` is a duration per request; `--timeout-ms` is an endpoint timeout, fixing the source map seconds/milliseconds mismatch and shadowed global flag. The source map `--wait` has no effect on its synchronous API and is omitted.

Single-format output selects the requested field, rather than an unrelated Markdown field returned alongside it. Timing always uses stderr. Cancel honors `--output`. Schema/action inline and file inputs are mutually exclusive. Output is atomically replaced with private permissions. Limits are validated before requests; documented CLI defaults of 100 map links and five search results are explicitly sent rather than silently adopting changing server defaults.

Current search documentation emphasizes `developer`, `research`, `pdf`, and `gov`; the actual server's [query builder](https://github.com/firecrawl/firecrawl/blob/main/apps/api/src/lib/search-query-builder.ts) also accepts `github`, meaning web search restricted to GitHub pages. The CLI preserves that source feature and offers `developer` as a distinct index, validating its exclusive-category constraint before API access. The server's [notice middleware](https://github.com/firecrawl/firecrawl/blob/main/apps/api/src/lib/deprecations.ts) adds string-array `warnings`; these are retained and displayed. The docs announce a research-response change on 2026-11-16; this pinned SDK's response model must be checked before using that future contract.

Coverage gates measure this package's `src/` source. The complete vendored third-party SDK is excluded from that package percentage; its unchanged endpoints are outside this CLI's scope. Local HTTP tests cover patched native behavior, and the SDK's own unit suite is run separately. The unfiltered first report is retained in `.tmp/coverage-first.json`.

The compiled skill pointer identifies the source repository's intended skill distribution path. The Rust executable and skill have not been published there. Live Firecrawl behavior and non-Linux runtime behavior require separate verification; local fixtures establish request/response contracts only.
