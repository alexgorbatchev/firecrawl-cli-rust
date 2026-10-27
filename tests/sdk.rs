use firecrawl::{Action, Client, Format, JsonOptions, ScrapeOptions, ScreenshotOptions};
use serde_json::json;

#[tokio::test]
async fn native_format_objects_reach_the_actual_api_request() {
    let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let address = format!("http://{}", server.server_addr());
    let worker = std::thread::spawn(move || {
        let mut request = server.recv().unwrap();
        assert_eq!(request.url(), "/v2/scrape");
        let body: serde_json::Value = serde_json::from_reader(request.as_reader()).unwrap();
        assert_eq!(
            body["formats"],
            json!([
                {"type":"json","prompt":"Extract prices","schema":{"type":"object"}},
                {"type":"screenshot","fullPage":true}
            ])
        );
        assert!(body.get("jsonOptions").is_none());
        assert!(body.get("screenshotOptions").is_none());
        assert_eq!(
            body["actions"],
            json!([{"type":"screenshot","fullPage":true}])
        );
        request.respond(tiny_http::Response::from_string(r#"{"success":true,"data":{"json":{"price":5},"screenshot":"https://example.com/image"}}"#)).unwrap();
    });
    let client = Client::new_selfhosted(address, None::<&str>).unwrap();
    let result = client
        .scrape(
            "https://example.com",
            ScrapeOptions {
                formats: Some(vec![
                    Format::JsonWithOptions(JsonOptions {
                        prompt: Some("Extract prices".into()),
                        schema: Some(json!({"type":"object"})),
                        ..Default::default()
                    }),
                    Format::ScreenshotWithOptions(ScreenshotOptions {
                        full_page: Some(true),
                        ..Default::default()
                    }),
                ]),
                actions: Some(
                    serde_json::from_value::<Vec<Action>>(
                        json!([{"type":"screenshot","fullPage":true}]),
                    )
                    .unwrap(),
                ),
                ..Default::default()
            },
        )
        .await;
    worker.join().unwrap();
    assert_eq!(result.unwrap().json, Some(json!({"price":5})));
}

#[tokio::test]
async fn native_client_rejects_http_failure_even_when_success_is_true() {
    let server = tiny_http::Server::http("127.0.0.1:0").unwrap();
    let address = format!("http://{}", server.server_addr());
    let worker = std::thread::spawn(move || {
        server
            .recv()
            .unwrap()
            .respond(
                tiny_http::Response::from_string(r#"{"success":true,"links":[]}"#)
                    .with_status_code(503),
            )
            .unwrap();
    });
    let result = Client::new_selfhosted(address, None::<&str>)
        .unwrap()
        .map("https://example.com", None)
        .await;
    worker.join().unwrap();
    assert!(
        result.is_err(),
        "an HTTP failure must never be reported as success"
    );
}
