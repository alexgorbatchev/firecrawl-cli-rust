use crate::{ApiError, args, options, output};
use anyhow::{Context, Result, bail, ensure};
use cli_command_kit::{Mode, clap::ArgMatches};
use firecrawl::{AgentStatus, AgentStatusResponse, Client};
use std::{future::Future, io::Write, time::Duration};
use tokio_util::sync::CancellationToken;

async fn request<T>(
    timeout: Duration,
    cancel: &CancellationToken,
    future: impl Future<Output = Result<T, firecrawl::FirecrawlError>>,
) -> Result<T> {
    tokio::select! {
        biased;
        _ = cancel.cancelled() => Err(ApiError::Cancelled.into()),
        result = tokio::time::timeout(timeout, future) => result.map_err(|_| ApiError::Timeout)?.map_err(Into::into),
    }
}

fn client(args: &ArgMatches, timeout: Duration) -> Result<Client> {
    let api_url = args
        .get_one::<String>("api-url")
        .context("set --api-url or FIRECRAWL_API_URL")?;
    let key = args.get_one::<String>("api-key");
    if let Some(key) = key {
        reqwest::header::HeaderValue::from_str(&format!("Bearer {key}"))
            .map_err(|_| anyhow::anyhow!("API key contains invalid header characters"))?;
    }
    let http = reqwest::Client::builder()
        .use_rustls_tls()
        .redirect(reqwest::redirect::Policy::none())
        .retry(reqwest::retry::never())
        .timeout(timeout)
        .build()?;
    Ok(Client::new_selfhosted(api_url, key)
        .context("creating Firecrawl client")?
        .with_http_client(http))
}

async fn wait(
    client: &Client,
    id: &str,
    args: &ArgMatches,
    timeout: Duration,
    cancel: &CancellationToken,
) -> Result<AgentStatusResponse> {
    let poll = *args.get_one::<Duration>("poll-interval").unwrap();
    let deadline = *args.get_one::<Duration>("wait-timeout").unwrap();
    let workflow = async {
        loop {
            let status = request(timeout, cancel, client.get_agent_status(id)).await?;
            ensure!(
                status.success,
                "status request failed: {}",
                status.error.as_deref().unwrap_or("no error details")
            );
            match status.status {
                AgentStatus::Completed => return Ok(status),
                AgentStatus::Failed | AgentStatus::Cancelled => bail!(
                    "agent task {:?}: {}",
                    status.status,
                    status.error.as_deref().unwrap_or("no error details")
                ),
                AgentStatus::Processing => {}
            }
            tokio::select! {
                biased;
                _ = cancel.cancelled() => return Err(ApiError::Cancelled.into()),
                _ = tokio::time::sleep(poll) => {}
            }
        }
    };
    tokio::time::timeout(deadline, workflow)
        .await
        .map_err(|_| ApiError::Timeout)?
}

pub async fn perform(
    matches: &ArgMatches,
    destination: &mut impl Write,
    diagnostics: &mut impl Write,
    mode: Mode,
    cancel: &CancellationToken,
) -> Result<()> {
    if cli_command_kit::write_skill(matches, &args::help_options(), destination)? {
        return Ok(());
    }
    if matches.get_flag("version") {
        writeln!(destination, "{}", env!("CARGO_PKG_VERSION"))?;
        return Ok(());
    }
    let (subject, group) = matches.subcommand().unwrap();
    let (verb, args) = group.subcommand().unwrap();
    if subject == "completion" {
        let shell: clap_complete::Shell = verb.parse().expect("validated completion shell");
        let mut bytes = Vec::new();
        clap_complete::generate(shell, &mut crate::command(), "firecrawl", &mut bytes);
        destination.write_all(&bytes)?;
        return Ok(());
    }
    if cancel.is_cancelled() {
        return Err(ApiError::Cancelled.into());
    }
    let timeout = *matches.get_one::<Duration>("request-timeout").unwrap();
    let as_json = mode == Mode::Agent || matches.get_flag("json");
    let pretty = mode != Mode::Agent && args.get_flag("pretty");
    // Prepare and validate local input before client creation or any remote mutation.
    let text = match (subject, verb) {
        ("page", "scrape") => {
            let options = options::scrape(args)?;
            let client = client(matches, timeout)?;
            let started = std::time::Instant::now();
            let document = request(
                timeout,
                cancel,
                client.scrape(args.get_one::<String>("URL").unwrap(), options.clone()),
            )
            .await?;
            if args.get_flag("timing") {
                writeln!(
                    diagnostics,
                    "Duration: {}",
                    humantime::format_duration(started.elapsed())
                )?;
            }
            output::warning(document.warning.as_deref(), diagnostics, mode)?;
            output::document(
                &document,
                options.formats.as_deref().unwrap(),
                as_json,
                pretty,
            )?
        }
        ("site", "map") => {
            let options = options::map(args)?;
            let client = client(matches, timeout)?;
            let data = request(
                timeout,
                cancel,
                client.map(args.get_one::<String>("URL").unwrap(), options),
            )
            .await?;
            ensure!(data.success, "mapping request was unsuccessful");
            output::warning(data.warning.as_deref(), diagnostics, mode)?;
            if as_json {
                output::json(&data, pretty)?
            } else {
                output::map(&data, args.get_flag("detailed"))
            }
        }
        ("web", "search") => {
            let options = options::search(args)?;
            let client = client(matches, timeout)?;
            let data = request(
                timeout,
                cancel,
                client.search(args.get_one::<String>("QUERY").unwrap(), options),
            )
            .await?;
            ensure!(data.success, "search request was unsuccessful");
            output::warning(data.warning.as_deref(), diagnostics, mode)?;
            for warning in data.warnings.iter().flatten() {
                output::warning(Some(warning), diagnostics, mode)?;
            }
            if as_json {
                output::json(&data, pretty)?
            } else {
                output::search(&data.data)
            }
        }
        ("agent", "create") => {
            let options = options::agent(args)?;
            let client = client(matches, timeout)?;
            let task = request(timeout, cancel, client.start_agent(options)).await
                .context("agent creation did not produce a confirmed job ID; the server may have created a task; inspect the server before retrying")?;
            ensure!(
                task.success && !task.id.is_empty(),
                "agent creation failed: {}",
                task.error.as_deref().unwrap_or("no task ID")
            );
            if args.get_flag("wait") {
                writeln!(diagnostics, "Job: {}", task.id)?;
                let status = wait(&client, &task.id, args, timeout, cancel)
                    .await
                    .with_context(|| {
                        format!(
                            "waiting for job {}; use agent status to inspect it",
                            task.id
                        )
                    })?;
                if as_json {
                    output::json(&status, pretty)?
                } else {
                    output::json(&status.data, true)?
                }
            } else if as_json {
                output::json(&task, pretty)?
            } else {
                format!("Job: {}", task.id)
            }
        }
        ("agent", "status") => {
            let client = client(matches, timeout)?;
            let status = request(
                timeout,
                cancel,
                client.get_agent_status(args.get_one::<String>("JOB_ID").unwrap()),
            )
            .await?;
            ensure!(
                status.success,
                "status request failed: {}",
                status.error.as_deref().unwrap_or("no error details")
            );
            if as_json {
                output::json(&status, pretty)?
            } else {
                format!(
                    "Status: {:?}\n{}",
                    status.status,
                    output::json(&status, true)?
                )
            }
        }
        ("agent", "cancel") => {
            let client = client(matches, timeout)?;
            let id = args.get_one::<String>("JOB_ID").unwrap();
            let success = request(timeout, cancel, client.cancel_agent(id)).await
                .with_context(|| format!("cancellation of job {id} is unconfirmed; inspect its status before retrying"))?;
            ensure!(success, "server did not confirm cancellation of job {id}");
            if as_json {
                output::json(
                    &serde_json::json!({"success":true,"id":id,"cancelled":true}),
                    pretty,
                )?
            } else {
                format!("Cancelled job {id}")
            }
        }
        _ => unreachable!("validated command tree"),
    };
    output::emit(args, &text, destination)
}
