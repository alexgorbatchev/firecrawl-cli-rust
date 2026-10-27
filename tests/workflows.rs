use cli_command_kit::Mode;
use serde_json::{Value, json};
use std::{
    ffi::OsString,
    io::{self, Write},
    time::Duration,
};
use tokio_util::sync::CancellationToken;

struct Fixture {
    url: String,
    worker: std::thread::JoinHandle<Vec<(String, String, Value)>>,
}

impl Fixture {
    fn new(responses: Vec<(u16, Value)>) -> Self {
        let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
        let url = format!("http://{}", server.server_addr());
        let worker = std::thread::spawn(move || {
            let mut requests = Vec::new();
            for (code, body) in responses {
                let mut request = server
                    .recv_timeout(Duration::from_secs(3))
                    .unwrap()
                    .expect("local API request");
                assert_eq!(
                    request
                        .headers()
                        .iter()
                        .find(|header| header.field.equiv("Authorization"))
                        .unwrap()
                        .value
                        .as_str(),
                    "Bearer fixture-key"
                );
                let payload = if request.body_length().unwrap_or(0) == 0 {
                    Value::Null
                } else {
                    serde_json::from_reader(request.as_reader()).unwrap()
                };
                requests.push((
                    request.method().to_string(),
                    request.url().to_owned(),
                    payload,
                ));
                request
                    .respond(
                        tiny_http::Response::from_string(body.to_string()).with_status_code(code),
                    )
                    .unwrap();
            }
            requests
        });
        Self { url, worker }
    }

    async fn run(&self, args: &[&str], mode: Mode) -> (u8, String, String) {
        let mut all = vec![
            "firecrawl",
            "--api-url",
            &self.url,
            "--api-key",
            "fixture-key",
        ];
        all.extend_from_slice(args);
        run(&all, mode, &CancellationToken::new()).await
    }
}

async fn run(args: &[&str], mode: Mode, cancel: &CancellationToken) -> (u8, String, String) {
    let mut output = Vec::new();
    let mut errors = Vec::new();
    let code = firecrawl_cli::entry(
        args.iter().map(OsString::from),
        &mut output,
        &mut errors,
        mode,
        cancel,
    )
    .await;
    (
        code,
        String::from_utf8(output).unwrap(),
        String::from_utf8(errors).unwrap(),
    )
}

#[tokio::test]
async fn all_scrape_options_reach_native_http_and_timing_stays_off_json_stdout() {
    let fixture = Fixture::new(vec![(
        200,
        json!({"success":true,"data":{"markdown":"Content","json":{"price":5},"screenshot":"image","warning":"source warning"}}),
    )]);
    let (code, output, errors) = fixture
        .run(
            &[
                "page",
                "scrape",
                "https://example.com",
                "--format",
                "json,screenshot",
                "--json-prompt",
                "Prices",
                "--schema",
                r#"{"type":"object"}"#,
                "--full-page-screenshot",
                "--only-main-content=false",
                "--include-tags",
                "main,article",
                "--exclude-tags",
                "nav,aside",
                "--wait-for",
                "42",
                "--timeout-ms",
                "1000",
                "--mobile",
                "--skip-tls-verification=false",
                "--remove-base64-images",
                "--block-ads=false",
                "--proxy",
                "stealth",
                "--max-age",
                "60",
                "--store-in-cache",
                "--lockdown=false",
                "--redact-pii",
                "--location-country",
                "de",
                "--location-languages",
                "de,en",
                "--actions",
                r#"[{"type":"wait","milliseconds":12}]"#,
                "--timing",
                "--pretty",
            ],
            Mode::Agent,
        )
        .await;
    assert_eq!(code, 0, "{errors}");
    assert_eq!(
        serde_json::from_str::<Value>(&output).unwrap()["json"]["price"],
        5
    );
    assert_eq!(output.lines().count(), 1);
    assert!(errors.contains("Duration:") && errors.contains("WARN: source warning"));
    let requests = fixture.worker.join().unwrap();
    assert_eq!(
        (&requests[0].0, &requests[0].1),
        (&"POST".into(), &"/v2/scrape".into())
    );
    let body = &requests[0].2;
    for (name, expected) in [
        ("onlyMainContent", json!(false)),
        ("includeTags", json!(["main", "article"])),
        ("excludeTags", json!(["nav", "aside"])),
        ("waitFor", json!(42)),
        ("timeout", json!(1000)),
        ("mobile", json!(true)),
        ("skipTlsVerification", json!(false)),
        ("removeBase64Images", json!(true)),
        ("blockAds", json!(false)),
        ("proxy", json!("stealth")),
        ("maxAge", json!(60)),
        ("storeInCache", json!(true)),
        ("lockdown", json!(false)),
        ("redactPII", json!(true)),
        ("location", json!({"country":"DE","languages":["de","en"]})),
    ] {
        assert_eq!(body[name], expected, "{name}");
    }
    assert_eq!(
        body["formats"],
        json!([{"type":"json","prompt":"Prices","schema":{"type":"object"}},{"type":"screenshot","fullPage":true}])
    );
    assert_eq!(body["actions"][0]["milliseconds"], 12);
    assert!(body.get("jsonOptions").is_none());
}

#[tokio::test]
async fn single_format_output_uses_the_requested_content_and_files_supply_native_options() {
    let dir = tempfile::tempdir_in(concat!(env!("CARGO_MANIFEST_DIR"), "/.tmp")).unwrap();
    let schema = dir.path().join("schema.json");
    let actions = dir.path().join("actions.json");
    std::fs::write(&schema, r#"{"type":"object"}"#).unwrap();
    std::fs::write(&actions, r#"[{"type":"wait","milliseconds":1}]"#).unwrap();
    for (flags, expected) in [
        (vec![], "Text"),
        (vec!["--html"], "<p>HTML</p>"),
        (vec!["--format", "rawHtml"], "<html>Raw</html>"),
        (vec!["--format", "json"], r#"{"n":1}"#),
        (vec!["--format", "screenshot"], "image"),
        (vec!["--format", "links"], r#"["https://example.com/link"]"#),
        (vec!["--format", "summary"], "Summary"),
        (vec!["--format", "changeTracking"], r#"{"changed":true}"#),
    ] {
        let fixture = Fixture::new(vec![(
            200,
            json!({"success":true,"data":{"markdown":"Text","html":"<p>HTML</p>","rawHtml":"<html>Raw</html>","json":{"n":1},"screenshot":"image","links":["https://example.com/link"],"summary":"Summary","changeTracking":{"changed":true}}}),
        )]);
        let mut args = vec!["page", "scrape", "https://example.com"];
        args.extend(flags);
        let result = fixture.run(&args, Mode::Human).await;
        assert_eq!(result, (0, format!("{expected}\n"), String::new()));
        fixture.worker.join().unwrap();
    }
    for flags in [
        vec![
            "--schema-file",
            schema.to_str().unwrap(),
            "--actions-file",
            actions.to_str().unwrap(),
        ],
        vec!["--json-prompt", "Extract", "--screenshot"],
        vec!["--full-page-screenshot"],
    ] {
        let fixture = Fixture::new(vec![(200, json!({"success":true,"data":{"json":{"n":1}}}))]);
        let mut args = vec!["page", "scrape", "https://example.com"];
        args.extend(flags);
        let result = fixture.run(&args, Mode::Human).await;
        assert_eq!(result.0, 0, "{}", result.2);
        let request = fixture.worker.join().unwrap().remove(0).2;
        assert!(request["formats"].as_array().unwrap().len() > 1);
    }
    let fixture = Fixture::new(vec![(
        200,
        json!({"success":true,"data":{"metadata":{"title":"Only metadata"}}}),
    )]);
    let result = fixture
        .run(&["page", "scrape", "https://example.com"], Mode::Human)
        .await;
    assert_eq!(result.0, 0);
    assert!(result.1.contains("Only metadata"));
    fixture.worker.join().unwrap();
}

#[tokio::test]
async fn map_options_have_correct_units_and_preserve_detailed_and_machine_output() {
    for mode in [Mode::Human, Mode::Agent] {
        let fixture = Fixture::new(vec![(
            200,
            json!({"success":true,"links":[{"url":"https://example.com/a","title":"Page A","description":"Description"}],"warning":"partial"}),
        )]);
        let result = fixture
            .run(
                &[
                    "site",
                    "map",
                    "https://example.com",
                    "--limit",
                    "14",
                    "--search",
                    "documentation",
                    "--sitemap",
                    "only",
                    "--include-subdomains",
                    "--ignore-query-parameters=false",
                    "--timeout-ms",
                    "1500",
                    "--location-country",
                    "us",
                    "--location-languages",
                    "en",
                    "--detailed",
                ],
                mode,
            )
            .await;
        assert_eq!(result.0, 0, "{}", result.2);
        assert!(result.1.contains("Page A") && result.2.contains("partial"));
        let body = fixture.worker.join().unwrap().remove(0).2;
        assert_eq!(body["timeout"], 1500);
        assert_eq!(body["limit"], 14);
        assert_eq!(body["search"], "documentation");
        assert_eq!(body["sitemap"], "only");
        assert_eq!(body["includeSubdomains"], true);
        assert_eq!(body["ignoreQueryParameters"], false);
        assert_eq!(body["location"], json!({"country":"US","languages":["en"]}));
    }
    for links in [json!([]), json!([{"url":"https://example.com/a"}])] {
        let fixture = Fixture::new(vec![(200, json!({"success":true,"links":links}))]);
        let result = fixture
            .run(&["site", "map", "https://example.com"], Mode::Human)
            .await;
        assert_eq!(result.0, 0);
        assert!(result.1 == "No URLs found.\n" || result.1 == "https://example.com/a\n");
        assert_eq!(fixture.worker.join().unwrap().remove(0).2["limit"], 100);
    }
}

#[tokio::test]
async fn search_country_timeout_categories_and_result_content_are_applied() {
    let data = json!({"web":[{"title":"Web title","url":"https://example.com","description":"Snippet"},{"markdown":"Actual content","metadata":{"sourceURL":"https://example.com/page"}}],"news":[{"title":"News title","url":"https://example.com/news","snippet":"News snippet"}],"images":[{"title":"Image title","url":"https://example.com/image","imageUrl":"https://example.com/photo"}]});
    for mode in [Mode::Human, Mode::Agent] {
        let fixture = Fixture::new(vec![(
            200,
            json!({"success":true,"data":data,"id":"search-job","creditsUsed":7,"warnings":["Research response contract changes soon."]}),
        )]);
        let result = fixture
            .run(
                &[
                    "web",
                    "search",
                    "Rust tooling",
                    "--country",
                    "de",
                    "--timeout-ms",
                    "12345",
                    "--limit",
                    "77",
                    "--sources",
                    "web,news,images",
                    "--categories",
                    "github,research,pdf",
                    "--tbs",
                    "qdr:w",
                    "--location",
                    "Berlin,Germany",
                    "--ignore-invalid-urls",
                    "--scrape",
                    "--scrape-formats",
                    "markdown,html",
                    "--scrape-only-main-content=false",
                ],
                mode,
            )
            .await;
        assert_eq!(result.0, 0, "{}", result.2);
        assert!(
            result
                .2
                .contains("Research response contract changes soon.")
        );
        if mode == Mode::Agent {
            let payload: Value = serde_json::from_str(&result.1).unwrap();
            assert_eq!(payload["id"], "search-job");
            assert_eq!(payload["creditsUsed"], 7);
            assert_eq!(
                payload["warnings"][0],
                "Research response contract changes soon."
            );
        }
        assert!(
            result.1.contains("Actual content")
                && result.1.contains("Image title")
                && result.1.contains("News title")
        );
        let body = fixture.worker.join().unwrap().remove(0).2;
        assert_eq!(body["country"], "DE");
        assert_eq!(body["timeout"], 12345);
        assert_eq!(body["limit"], 77);
        assert_eq!(body["sources"], json!(["web", "news", "images"]));
        assert_eq!(body["categories"], json!(["github", "research", "pdf"]));
        assert_eq!(body["tbs"], "qdr:w");
        assert_eq!(body["location"], "Berlin,Germany");
        assert_eq!(body["ignoreInvalidURLs"], true);
        assert_eq!(body["scrapeOptions"]["onlyMainContent"], false);
        assert_eq!(
            body["scrapeOptions"]["formats"],
            json!(["markdown", "html"])
        );
    }
    let fixture = Fixture::new(vec![(200, json!({"success":true,"data":{}}))]);
    assert_eq!(
        fixture.run(&["web", "search", "Empty"], Mode::Human).await,
        (0, "No search results found.\n".into(), String::new())
    );
    let body = fixture.worker.join().unwrap().remove(0).2;
    assert_eq!(body["country"], "US");
    assert_eq!(body["timeout"], 60000);
    assert_eq!(body["limit"], 5);
    assert!(body.get("scrapeOptions").is_none());
    let fixture = Fixture::new(vec![(
        200,
        json!({"success":true,"data":{"web":[{"html":"<b>HTML</b>"}]}}),
    )]);
    let result = fixture
        .run(&["web", "search", "HTML", "--scrape"], Mode::Human)
        .await;
    assert!(result.1.contains("<b>HTML</b>"));
    assert_eq!(
        fixture.worker.join().unwrap().remove(0).2["scrapeOptions"]["formats"],
        json!(["markdown"])
    );
    let fixture = Fixture::new(vec![(200, json!({"success":true,"data":{}}))]);
    assert_eq!(
        fixture
            .run(
                &["web", "search", "Code", "--categories", "developer"],
                Mode::Agent
            )
            .await
            .0,
        0
    );
    assert_eq!(
        fixture.worker.join().unwrap().remove(0).2["categories"],
        json!(["developer"])
    );
}

#[tokio::test]
async fn agent_starts_once_waits_explicitly_and_retains_the_recovery_id() {
    for mode in [Mode::Human, Mode::Agent] {
        let fixture = Fixture::new(vec![(200, json!({"success":true,"id":"task-123"}))]);
        let result = fixture
            .run(
                &[
                    "agent",
                    "create",
                    "Find prices",
                    "--urls",
                    "https://example.com,https://example.org",
                    "--schema",
                    r#"{"type":"object"}"#,
                    "--max-credits",
                    "12",
                    "--strict-constrain-to-urls",
                    "--model",
                    "spark-2",
                    "--webhook",
                    r#"{"url":"https://example.com/hook","events":["completed"]}"#,
                ],
                mode,
            )
            .await;
        assert_eq!(result.0, 0, "{}", result.2);
        assert!(result.1.contains("task-123"));
        assert!(result.2.is_empty());
        let body = fixture.worker.join().unwrap().remove(0).2;
        assert_eq!(body["maxCredits"], 12);
        assert_eq!(body["strictConstrainToURLs"], true);
        assert_eq!(body["model"], "spark-2");
        assert_eq!(body["schema"], json!({"type":"object"}));
        assert_eq!(body["webhook"]["events"], json!(["completed"]));
        assert_eq!(body["urls"].as_array().unwrap().len(), 2);
    }
    for mode in [Mode::Human, Mode::Agent] {
        let fixture = Fixture::new(vec![
            (200, json!({"success":true,"id":"task-123"})),
            (200, json!({"success":true,"status":"processing"})),
            (
                200,
                json!({"success":true,"status":"completed","data":{"price":5}}),
            ),
        ]);
        let result = fixture
            .run(
                &[
                    "agent",
                    "create",
                    "Find prices",
                    "--wait",
                    "--poll-interval",
                    "1ms",
                    "--wait-timeout",
                    "2s",
                    "--webhook",
                    "https://example.com/hook",
                ],
                mode,
            )
            .await;
        assert_eq!(result.0, 0, "{}", result.2);
        assert!(result.1.contains("5"));
        assert_eq!(result.2, "Job: task-123\n");
        let requests = fixture.worker.join().unwrap();
        assert_eq!(requests.len(), 3);
        assert_eq!(requests[1].0, "GET");
        assert_eq!(requests[2].1, "/v2/agent/task-123");
    }
}

#[tokio::test]
async fn agent_failures_cancel_and_timeout_have_truthful_outcomes() {
    for response in [
        json!({"success":true,"status":"failed","error":"task failed"}),
        json!({"success":true,"status":"cancelled"}),
        json!({"success":false,"status":"processing","error":"status denied"}),
        json!({"success":true,"status":"processing"}),
    ] {
        let fixture = Fixture::new(vec![
            (200, json!({"success":true,"id":"task-123"})),
            (200, response),
        ]);
        let result = fixture
            .run(
                &[
                    "agent",
                    "create",
                    "Task",
                    "--wait",
                    "--poll-interval",
                    "5s",
                    "--wait-timeout",
                    "30ms",
                ],
                Mode::Agent,
            )
            .await;
        assert_eq!(result.0, 1);
        assert!(result.1.is_empty());
        assert!(result.2.contains("waiting for job task-123"));
        assert!(result.2.starts_with("Job: task-123\nERR:"));
        fixture.worker.join().unwrap();
    }
    for mode in [Mode::Human, Mode::Agent] {
        let fixture = Fixture::new(vec![
            (
                200,
                json!({"success":true,"status":"completed","data":{"done":true}}),
            ),
            (200, json!({"success":true})),
        ]);
        assert_eq!(
            fixture.run(&["agent", "status", "task-123"], mode).await.0,
            0
        );
        let result = fixture.run(&["agent", "cancel", "task-123"], mode).await;
        assert_eq!(result.0, 0);
        assert!(result.1.contains("task-123"));
        let requests = fixture.worker.join().unwrap();
        assert_eq!(requests[1].0, "DELETE");
    }
    for (verb, response) in [
        ("cancel", json!({"success":false})),
        (
            "status",
            json!({"success":false,"status":"processing","error":"denied"}),
        ),
        ("create", json!({"success":true,"id":""})),
    ] {
        let fixture = Fixture::new(vec![(200, response)]);
        let result = fixture.run(&["agent", verb, "task-123"], Mode::Agent).await;
        assert_eq!(result.0, 1);
        assert!(result.1.is_empty());
        assert_eq!(result.2.lines().count(), 1);
        fixture.worker.join().unwrap();
    }
}

#[tokio::test]
async fn output_files_are_atomic_private_and_cancel_honors_output() {
    let dir = tempfile::tempdir_in(concat!(env!("CARGO_MANIFEST_DIR"), "/.tmp")).unwrap();
    let output = dir.path().join("result.json");
    std::fs::write(&output, "old").unwrap();
    let fixture = Fixture::new(vec![(200, json!({"success":true}))]);
    let result = fixture
        .run(
            &[
                "agent",
                "cancel",
                "task-123",
                "--output",
                output.to_str().unwrap(),
                "--pretty",
            ],
            Mode::Agent,
        )
        .await;
    assert_eq!(result, (0, String::new(), String::new()));
    assert_eq!(
        serde_json::from_slice::<Value>(&std::fs::read(&output).unwrap()).unwrap()["cancelled"],
        true
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            std::fs::metadata(&output).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
    fixture.worker.join().unwrap();
    let fixture = Fixture::new(vec![(200, json!({"success":true,"links":[]}))]);
    let result = fixture
        .run(
            &[
                "site",
                "map",
                "https://example.com",
                "--output",
                dir.path().to_str().unwrap(),
            ],
            Mode::Agent,
        )
        .await;
    assert_eq!(result.0, 1);
    assert!(dir.path().is_dir());
    fixture.worker.join().unwrap();
    let missing = dir.path().join("missing/output");
    let fixture = Fixture::new(vec![(200, json!({"success":true,"links":[]}))]);
    assert_eq!(
        fixture
            .run(
                &[
                    "site",
                    "map",
                    "https://example.com",
                    "--output",
                    missing.to_str().unwrap()
                ],
                Mode::Agent
            )
            .await
            .0,
        1
    );
    fixture.worker.join().unwrap();
}

#[tokio::test]
async fn transport_failures_and_redirects_do_not_replay_mutations() {
    for (subject, verb, arg) in [
        ("page", "scrape", "https://example.com"),
        ("site", "map", "https://example.com"),
        ("web", "search", "query"),
        ("agent", "create", "Task"),
        ("agent", "cancel", "task-123"),
    ] {
        let fixture = Fixture::new(vec![(
            503,
            json!({"success":true,"links":[],"data":{},"id":"task-123"}),
        )]);
        let result = fixture.run(&[subject, verb, arg], Mode::Agent).await;
        assert_eq!(result.0, 1);
        assert!(result.1.is_empty());
        assert_eq!(result.2.lines().count(), 1);
        assert_eq!(fixture.worker.join().unwrap().len(), 1);
    }
    let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let url = format!("http://{}", server.server_addr());
    let worker = std::thread::spawn(move || {
        let request = server
            .recv_timeout(Duration::from_secs(3))
            .unwrap()
            .unwrap();
        request
            .respond(
                tiny_http::Response::from_string(r#"{"success":true,"id":"unexpected"}"#)
                    .with_status_code(307)
                    .with_header(
                        tiny_http::Header::from_bytes("Location", "/v2/agent/replayed").unwrap(),
                    ),
            )
            .unwrap();
        assert!(
            server
                .recv_timeout(Duration::from_millis(50))
                .unwrap()
                .is_none()
        );
    });
    let result = run(
        &["firecrawl", "--api-url", &url, "agent", "create", "Task"],
        Mode::Agent,
        &CancellationToken::new(),
    )
    .await;
    assert_eq!(result.0, 1);
    assert!(result.2.contains("may have created"));
    worker.join().unwrap();
}

#[tokio::test]
async fn deadlines_and_native_cancellation_stop_inflight_requests() {
    for cancel_request in [false, true] {
        let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
        let url = format!("http://{}", server.server_addr());
        let worker = std::thread::spawn(move || {
            let request = server
                .recv_timeout(Duration::from_secs(3))
                .unwrap()
                .unwrap();
            std::thread::sleep(Duration::from_millis(100));
            let _ = request.respond(tiny_http::Response::from_string(
                r#"{"success":true,"links":[]}"#,
            ));
        });
        let token = CancellationToken::new();
        let signal = token.clone();
        if cancel_request {
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(30)).await;
                signal.cancel();
            });
        }
        let result = run(
            &[
                "firecrawl",
                "--api-url",
                &url,
                "--request-timeout",
                if cancel_request { "2s" } else { "20ms" },
                "site",
                "map",
                "https://example.com",
            ],
            Mode::Agent,
            &token,
        )
        .await;
        assert_eq!(result.0, if cancel_request { 130 } else { 1 });
        assert!(result.1.is_empty());
        assert_eq!(result.2.lines().count(), 1);
        worker.join().unwrap();
    }
    let token = CancellationToken::new();
    token.cancel();
    assert_eq!(
        run(
            &["firecrawl", "site", "map", "https://example.com"],
            Mode::Agent,
            &token
        )
        .await
        .0,
        130
    );
}

struct Broken;
impl Write for Broken {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("writer unavailable"))
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[tokio::test]
async fn metadata_and_input_validation_stay_offline_and_handle_output_failures() {
    for mode in [Mode::Human, Mode::Agent] {
        assert_eq!(
            run(&["firecrawl", "--version"], mode, &CancellationToken::new()).await,
            (0, "0.1.0\n".into(), String::new())
        );
        let skill = run(&["firecrawl", "skill"], mode, &CancellationToken::new()).await;
        assert_eq!(skill.0, 0);
        assert!(
            skill
                .1
                .contains("npx skills add alexgorbatchev/firecrawl-cli --skill firecrawl")
        );
        for args in [
            vec!["firecrawl", "skill"],
            vec!["firecrawl", "--version"],
            vec!["firecrawl", "--help"],
            vec!["firecrawl", "completion", "fish"],
        ] {
            let mut errors = Vec::new();
            assert_eq!(
                firecrawl_cli::entry(
                    args.iter().map(OsString::from),
                    &mut Broken,
                    &mut errors,
                    mode,
                    &CancellationToken::new()
                )
                .await,
                1
            );
            assert_eq!(String::from_utf8(errors).unwrap().lines().count(), 1);
        }
    }
    for extra in [
        vec!["--format", "bad"],
        vec!["--schema", "[]"],
        vec!["--schema", "{}", "--schema-file", "file"],
        vec!["--actions", "[1]"],
        vec!["--actions", "[]", "--actions-file", "file"],
        vec!["--timeout-ms", "0"],
        vec!["--location-country", "USA"],
        vec!["--proxy", "invalid"],
        vec!["--html", "--format", "html"],
    ] {
        let mut args = vec!["firecrawl", "page", "scrape", "https://example.com"];
        args.extend(extra);
        let result = run(&args, Mode::Agent, &CancellationToken::new()).await;
        assert_eq!(result.0, 1);
        assert!(result.1.is_empty());
        assert!(result.2.contains("error:"));
    }
    for args in [
        vec!["firecrawl", "skill", "extra"],
        vec!["firecrawl", "site", "map", "ftp://example.com"],
        vec![
            "firecrawl",
            "--api-url",
            "https://example.com?token=bad",
            "site",
            "map",
            "https://example.com",
        ],
        vec!["firecrawl", "web", "search", " "],
        vec!["firecrawl", "agent", "status", "../outside"],
        vec![
            "firecrawl",
            "agent",
            "create",
            "Task",
            "--poll-interval",
            "0s",
        ],
        vec!["firecrawl", "agent", "create", "Task", "--webhook", "bad"],
        vec![
            "firecrawl",
            "agent",
            "create",
            "Task",
            "--strict-constrain-to-urls",
        ],
        vec!["firecrawl", "--version", "page"],
        vec!["firecrawl", "web", "search", "query", "--limit", "101"],
        vec![
            "firecrawl",
            "web",
            "search",
            "query",
            "--categories",
            "developer,research",
        ],
        vec![
            "firecrawl",
            "web",
            "search",
            "query",
            "--categories",
            "pdf",
            "--categories",
            "developer",
        ],
    ] {
        let result = run(&args, Mode::Human, &CancellationToken::new()).await;
        assert_eq!(result.0, 1, "{args:?}");
        assert!(result.1.is_empty());
        assert!(result.2.contains("error:"), "{args:?}: {}", result.2);
    }
    let result = run(
        &[
            "firecrawl",
            "--api-url",
            "http://127.0.0.1:1",
            "--api-key",
            "SECRET\nKEY",
            "site",
            "map",
            "https://example.com",
        ],
        Mode::Agent,
        &CancellationToken::new(),
    )
    .await;
    assert_eq!(result.0, 1);
    assert!(!result.2.contains("SECRET"));
    let result = std::process::Command::new(env!("CARGO_BIN_EXE_firecrawl"))
        .args(["site", "map", "https://example.com"])
        .env_remove("FIRECRAWL_API_URL")
        .env_remove("FIRECRAWL_API_KEY")
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(1));
    assert!(
        String::from_utf8(result.stderr)
            .unwrap()
            .contains("set --api-url")
    );
}

#[tokio::test]
async fn input_files_are_validated_before_remote_work_and_existing_output_is_preserved() {
    let dir = tempfile::tempdir_in(concat!(env!("CARGO_MANIFEST_DIR"), "/.tmp")).unwrap();
    let path = dir.path().join("input.json");
    for (flag, content) in [
        ("--schema-file", "[]"),
        ("--schema-file", "bad"),
        ("--actions-file", "[1]"),
    ] {
        std::fs::write(&path, content).unwrap();
        let result = run(
            &[
                "firecrawl",
                "--api-url",
                "http://127.0.0.1:1",
                "page",
                "scrape",
                "https://example.com",
                flag,
                path.to_str().unwrap(),
            ],
            Mode::Agent,
            &CancellationToken::new(),
        )
        .await;
        assert_eq!(result.0, 1);
        assert!(!result.2.contains("error sending request"));
        assert_eq!(std::fs::read_to_string(&path).unwrap(), content);
    }
    std::fs::remove_file(&path).unwrap();
    for flag in ["--schema-file", "--actions-file"] {
        assert_eq!(
            run(
                &[
                    "firecrawl",
                    "page",
                    "scrape",
                    "https://example.com",
                    flag,
                    path.to_str().unwrap()
                ],
                Mode::Agent,
                &CancellationToken::new()
            )
            .await
            .0,
            1
        );
    }
}

#[cfg(unix)]
#[test]
fn compiled_binary_handles_native_sigint_during_an_actual_http_request() {
    let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let url = format!("http://{}", server.server_addr());
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_firecrawl"))
        .env("AGENT", "1")
        .args([
            "--api-url",
            &url,
            "--api-key",
            "fixture-key",
            "page",
            "scrape",
            "https://example.com",
        ])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let request = server
        .recv_timeout(Duration::from_secs(3))
        .unwrap()
        .unwrap();
    assert_eq!(request.url(), "/v2/scrape");
    let sent = std::process::Command::new("kill")
        .args(["-INT", &child.id().to_string()])
        .status()
        .unwrap();
    assert!(sent.success());
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    while child.try_wait().unwrap().is_none() {
        if std::time::Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("SIGINT did not cancel the actual HTTP request");
        }
        std::thread::yield_now();
    }
    let result = child.wait_with_output().unwrap();
    assert_eq!(result.status.code(), Some(130));
    assert!(result.stdout.is_empty());
    assert_eq!(
        String::from_utf8(result.stderr).unwrap(),
        "ERR: operation cancelled\n"
    );
}
