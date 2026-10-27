use anyhow::{Context, Result, ensure};
use cli_command_kit::{
    HelpOptions,
    clap::{Arg, ArgAction, Command, value_parser},
};
use std::time::Duration;

pub const SKILL: &str = "Install the firecrawl skill using the Skills CLI:\n\n  npx skills add alexgorbatchev/firecrawl-cli --skill firecrawl\n\nSkill:\n\n  https://github.com/alexgorbatchev/firecrawl-cli/tree/main/skills/firecrawl\n";

pub fn help_options() -> HelpOptions {
    HelpOptions {
        skill: Some(SKILL.into()),
        ..Default::default()
    }
}

fn option(name: &'static str, help: &'static str) -> Arg {
    Arg::new(name).long(name).help(help)
}

fn flag(name: &'static str, help: &'static str) -> Arg {
    option(name, help).action(ArgAction::SetTrue)
}

fn boolean(name: &'static str, help: &'static str) -> Arg {
    option(name, help)
        .num_args(0..=1)
        .default_missing_value("true")
        .value_parser(value_parser!(bool))
        .require_equals(true)
}

fn list(name: &'static str, help: &'static str) -> Arg {
    option(name, help)
        .value_delimiter(',')
        .action(ArgAction::Append)
}

fn output(command: Command) -> Command {
    command
        .arg(
            option("output", "Save the result atomically to a file")
                .value_parser(value_parser!(std::path::PathBuf)),
        )
        .arg(flag("pretty", "Indent JSON output"))
}

fn schema(command: Command) -> Command {
    command
        .arg(
            option("schema", "Inline JSON object schema")
                .conflicts_with("schema-file")
                .value_parser(parse_schema),
        )
        .arg(
            option("schema-file", "Read a JSON object schema from a file")
                .value_parser(value_parser!(std::path::PathBuf)),
        )
}

pub fn parse_schema(value: &str) -> std::result::Result<serde_json::Value, String> {
    let schema: serde_json::Value =
        serde_json::from_str(value).map_err(|error| error.to_string())?;
    if schema.is_object() {
        Ok(schema)
    } else {
        Err("schema must be a JSON object".into())
    }
}

pub fn duration(value: &str) -> std::result::Result<Duration, String> {
    humantime::parse_duration(value)
        .map_err(|error| error.to_string())
        .and_then(|duration| {
            if duration.is_zero() {
                Err("duration must be positive".into())
            } else {
                Ok(duration)
            }
        })
}

pub fn http_url(value: &str) -> std::result::Result<String, String> {
    let parsed = url::Url::parse(value).map_err(|error| error.to_string())?;
    if matches!(parsed.scheme(), "http" | "https")
        && parsed.has_host()
        && parsed.username().is_empty()
        && parsed.password().is_none()
    {
        Ok(value.into())
    } else {
        Err("use an absolute HTTP or HTTPS URL without credentials".into())
    }
}

fn api_url(value: &str) -> std::result::Result<String, String> {
    let value = http_url(value)?;
    let parsed = url::Url::parse(&value).map_err(|error| error.to_string())?;
    if parsed.query().is_some() || parsed.fragment().is_some() {
        Err("API base URL must have no query or fragment".into())
    } else {
        Ok(value)
    }
}

fn country(value: &str) -> std::result::Result<String, String> {
    if value.len() == 2 && value.bytes().all(|byte| byte.is_ascii_alphabetic()) {
        Ok(value.to_ascii_uppercase())
    } else {
        Err("country must be a two-letter ISO country code".into())
    }
}

fn job_id(value: &str) -> std::result::Result<String, String> {
    if !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        Ok(value.into())
    } else {
        Err("job ID must contain 1–128 ASCII letters, digits, underscores, or hyphens".into())
    }
}

fn formats(name: &'static str, help: &'static str) -> Arg {
    list(name, help).value_parser([
        "markdown",
        "html",
        "rawHtml",
        "links",
        "screenshot",
        "json",
        "images",
        "summary",
        "changeTracking",
        "attributes",
        "branding",
    ])
}

pub fn validate(
    command: &mut Command,
    matches: cli_command_kit::clap::ArgMatches,
) -> Result<cli_command_kit::clap::ArgMatches, cli_command_kit::clap::Error> {
    if let Some(("web", group)) = matches.subcommand()
        && let Some(("search", args)) = group.subcommand()
        && let Some(values) = args.get_many::<String>("categories")
    {
        let categories = values.collect::<Vec<_>>();
        if categories.iter().any(|value| *value == "developer")
            && categories.iter().any(|value| *value != "developer")
        {
            return Err(command.error(
                cli_command_kit::clap::error::ErrorKind::ArgumentConflict,
                "--categories developer cannot be combined with other search categories",
            ));
        }
    }
    Ok(matches)
}

pub fn command() -> Command {
    let scrape = schema(output(
        Command::new("scrape")
            .about("Extract a page's content")
            .arg(
                Arg::new("URL")
                    .required(true)
                    .value_parser(http_url)
                    .help("Page URL"),
            )
            .arg(formats("format", "Comma-separated output formats").conflicts_with("html"))
            .arg(flag("html", "Return HTML instead of Markdown"))
            .arg(
                boolean("only-main-content", "Extract the main page content").default_value("true"),
            )
            .arg(list("include-tags", "Comma-separated tags to include"))
            .arg(list("exclude-tags", "Comma-separated tags to exclude"))
            .arg(
                option("wait-for", "Wait after page load, in milliseconds")
                    .value_parser(value_parser!(u32)),
            )
            .arg(
                option("timeout-ms", "Server scrape timeout, in milliseconds")
                    .value_parser(value_parser!(u32).range(1000..=300000)),
            )
            .arg(boolean("mobile", "Use a mobile browser"))
            .arg(boolean(
                "skip-tls-verification",
                "Skip the target page's TLS certificate checks",
            ))
            .arg(boolean(
                "remove-base64-images",
                "Remove embedded image data",
            ))
            .arg(boolean("block-ads", "Block advertisements"))
            .arg(
                option("proxy", "Select the page proxy").value_parser(["auto", "basic", "stealth"]),
            )
            .arg(
                option("max-age", "Maximum cached content age, in seconds")
                    .value_parser(value_parser!(u32)),
            )
            .arg(boolean(
                "store-in-cache",
                "Store the scraped result in the server cache",
            ))
            .arg(boolean(
                "lockdown",
                "Return cached content without outbound page requests",
            ))
            .arg(boolean("redact-pii", "Redact personal information"))
            .arg(
                option("location-country", "Two-letter country for page requests")
                    .value_parser(country),
            )
            .arg(list(
                "location-languages",
                "Comma-separated preferred languages",
            ))
            .arg(option(
                "json-prompt",
                "Describe the structured data to extract",
            ))
            .arg(
                option("actions", "Inline browser actions JSON array")
                    .value_parser(parse_actions)
                    .conflicts_with("actions-file"),
            )
            .arg(
                option("actions-file", "Read browser actions from a JSON file")
                    .value_parser(value_parser!(std::path::PathBuf)),
            )
            .arg(flag("screenshot", "Include a screenshot"))
            .arg(flag(
                "full-page-screenshot",
                "Include a screenshot of the entire page",
            ))
            .arg(flag("timing", "Report elapsed request time on stderr")),
    ));
    let map = output(
        Command::new("map")
            .about("Discover a site's URLs")
            .arg(
                Arg::new("URL")
                    .required(true)
                    .value_parser(http_url)
                    .help("Site URL"),
            )
            .arg(option("search", "Filter links by a search query"))
            .arg(
                option("sitemap", "Choose how to use the site's sitemap")
                    .value_parser(["skip", "include", "only"]),
            )
            .arg(boolean("include-subdomains", "Include links on subdomains"))
            .arg(boolean(
                "ignore-query-parameters",
                "Deduplicate URLs without their query strings",
            ))
            .arg(
                option("limit", "Maximum number of discovered links")
                    .default_value("100")
                    .value_parser(value_parser!(u32).range(1..=100000)),
            )
            .arg(
                option("timeout-ms", "Server mapping timeout, in milliseconds")
                    .value_parser(value_parser!(u32).range(1..)),
            )
            .arg(
                option("location-country", "Two-letter country for discovery")
                    .value_parser(country),
            )
            .arg(list(
                "location-languages",
                "Comma-separated preferred languages",
            ))
            .arg(flag("detailed", "Include link titles and descriptions")),
    );
    let search = output(
        Command::new("search")
            .about("Search the web and optionally read the results")
            .arg(
                Arg::new("QUERY")
                    .required(true)
                    .help("Search query")
                    .value_parser(query),
            )
            .arg(
                option("limit", "Maximum number of search results")
                    .default_value("5")
                    .value_parser(value_parser!(u32).range(1..=100)),
            )
            .arg(
                list("sources", "Comma-separated search sources")
                    .value_parser(["web", "images", "news"]),
            )
            .arg(
                list("categories", "Comma-separated search categories").value_parser([
                    "github",
                    "developer",
                    "research",
                    "pdf",
                ]),
            )
            .arg(option("tbs", "Time filter, such as qdr:d or qdr:w"))
            .arg(option("location", "Geographic search location"))
            .arg(
                option("country", "Two-letter country for search")
                    .default_value("US")
                    .value_parser(country),
            )
            .arg(
                option("timeout-ms", "Server search timeout, in milliseconds")
                    .default_value("60000")
                    .value_parser(value_parser!(u32).range(1..)),
            )
            .arg(boolean(
                "ignore-invalid-urls",
                "Exclude invalid result URLs",
            ))
            .arg(flag("scrape", "Extract content from search results"))
            .arg(
                formats(
                    "scrape-formats",
                    "Comma-separated formats for result content",
                )
                .requires("scrape"),
            )
            .arg(
                boolean(
                    "scrape-only-main-content",
                    "Extract only main content from results",
                )
                .default_value("true"),
            ),
    );
    let create = schema(output(
        Command::new("create")
            .about("Start an autonomous web task")
            .arg(Arg::new("PROMPT").required(true).help("Task instructions"))
            .arg(list("urls", "Comma-separated starting URLs").value_parser(http_url))
            .arg(
                option("max-credits", "Maximum credits for this task")
                    .value_parser(value_parser!(u32).range(1..)),
            )
            .arg(
                boolean(
                    "strict-constrain-to-urls",
                    "Constrain the task to its starting URLs",
                )
                .requires("urls"),
            )
            .arg(option("model", "Choose the agent model").value_parser([
                "spark-2",
                "spark-1-pro",
                "spark-1-mini",
            ]))
            .arg(option("webhook", "Notification URL or webhook JSON object").value_parser(webhook))
            .arg(flag("wait", "Poll until the task finishes"))
            .arg(
                option("poll-interval", "Time between status requests")
                    .default_value("5s")
                    .value_parser(duration),
            )
            .arg(
                option("wait-timeout", "Maximum total wait for task completion")
                    .default_value("5m")
                    .value_parser(duration),
            ),
    ));
    let status = output(
        Command::new("status")
            .about("Read a task's current status")
            .arg(
                Arg::new("JOB_ID")
                    .required(true)
                    .value_parser(job_id)
                    .help("Agent task ID"),
            ),
    );
    let cancel = output(
        Command::new("cancel").about("Cancel an agent task").arg(
            Arg::new("JOB_ID")
                .required(true)
                .value_parser(job_id)
                .help("Agent task ID"),
        ),
    );
    let group = |name, about| Command::new(name).about(about).subcommand_required(true);
    let completion = ["bash", "zsh", "fish", "powershell", "elvish"]
        .into_iter()
        .fold(
            group("completion", "Generate shell completion scripts"),
            |command, shell| {
                command
                    .subcommand(Command::new(shell).about("Print this shell's completion script"))
            },
        );
    Command::new("firecrawl")
        .about("Read web pages and manage Firecrawl agent tasks")
        .arg_required_else_help(true)
        .arg(flag("version", "Print the raw version").exclusive(true))
        .arg(
            option("api-key", "Firecrawl API key")
                .global(true)
                .env("FIRECRAWL_API_KEY")
                .hide_env_values(true),
        )
        .arg(
            option("api-url", "Firecrawl API base URL")
                .global(true)
                .env("FIRECRAWL_API_URL")
                .value_parser(api_url),
        )
        .arg(
            option("request-timeout", "Maximum time for each API request")
                .global(true)
                .default_value("5m")
                .value_parser(duration),
        )
        .arg(flag("json", "Return JSON output").global(true))
        .subcommand(group("page", "Read individual pages").subcommand(scrape))
        .subcommand(group("site", "Discover website links").subcommand(map))
        .subcommand(group("web", "Find web results").subcommand(search))
        .subcommand(
            group("agent", "Manage autonomous web tasks").subcommands([create, status, cancel]),
        )
        .subcommand(completion)
}

#[derive(Clone, Debug)]
pub struct Actions(pub Vec<firecrawl::Action>);

pub fn parse_actions(value: &str) -> std::result::Result<Actions, String> {
    serde_json::from_str(value)
        .map(Actions)
        .map_err(|error| error.to_string())
}

fn query(value: &str) -> std::result::Result<String, String> {
    if !value.trim().is_empty() && value.chars().count() <= 500 {
        Ok(value.into())
    } else {
        Err("query must contain 1–500 characters".into())
    }
}

fn webhook(value: &str) -> std::result::Result<firecrawl::AgentWebhookConfig, String> {
    let config: firecrawl::AgentWebhookConfig = if value.trim_start().starts_with('{') {
        serde_json::from_str(value).map_err(|error| error.to_string())?
    } else {
        value.into()
    };
    http_url(&config.url)?;
    Ok(config)
}

pub fn file_schema(path: &std::path::Path) -> Result<serde_json::Value> {
    let data = std::fs::read_to_string(path)
        .with_context(|| format!("reading schema {}", path.display()))?;
    let value: serde_json::Value = serde_json::from_str(&data).context("parsing schema file")?;
    ensure!(value.is_object(), "schema must be a JSON object");
    Ok(value)
}
