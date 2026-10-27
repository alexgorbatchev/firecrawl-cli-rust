use anyhow::{Context, Result};
use cli_command_kit::clap::ArgMatches;
use firecrawl::{
    AgentOptions, Format, JsonOptions, LocationConfig, MapOptions, ScrapeOptions,
    ScreenshotOptions, SearchOptions,
};
use serde::de::DeserializeOwned;

pub fn one<T: Clone + Send + Sync + 'static>(args: &ArgMatches, name: &str) -> Option<T> {
    args.get_one::<T>(name).cloned()
}

pub fn strings(args: &ArgMatches, name: &str) -> Option<Vec<String>> {
    args.get_many::<String>(name)
        .map(|values| values.cloned().collect())
}

fn native<T: DeserializeOwned>(value: &str) -> Result<T> {
    serde_json::from_value(serde_json::Value::String(value.into()))
        .context("decoding native SDK option")
}

fn native_one<T: DeserializeOwned>(args: &ArgMatches, name: &str) -> Result<Option<T>> {
    args.get_one::<String>(name)
        .map(|value| native(value))
        .transpose()
}

fn native_list<T: DeserializeOwned>(args: &ArgMatches, name: &str) -> Result<Option<Vec<T>>> {
    args.get_many::<String>(name)
        .map(|values| values.map(|value| native(value)).collect())
        .transpose()
}

fn location(args: &ArgMatches) -> Option<LocationConfig> {
    let country = one(args, "location-country");
    let languages = strings(args, "location-languages");
    (country.is_some() || languages.is_some()).then_some(LocationConfig { country, languages })
}

fn schema(args: &ArgMatches) -> Result<Option<serde_json::Value>> {
    if let Some(path) = args.get_one::<std::path::PathBuf>("schema-file") {
        crate::args::file_schema(path).map(Some)
    } else {
        Ok(one(args, "schema"))
    }
}

pub fn scrape(args: &ArgMatches) -> Result<ScrapeOptions> {
    let mut formats = native_list::<Format>(args, "format")?.unwrap_or_else(|| {
        vec![if args.get_flag("html") {
            Format::Html
        } else {
            Format::Markdown
        }]
    });
    let json = JsonOptions {
        prompt: one(args, "json-prompt"),
        schema: schema(args)?,
        ..Default::default()
    };
    if json.prompt.is_some() || json.schema.is_some() {
        if let Some(format) = formats.iter_mut().find(|format| **format == Format::Json) {
            *format = Format::JsonWithOptions(json);
        } else {
            formats.push(Format::JsonWithOptions(json));
        }
    }
    if args.get_flag("full-page-screenshot") {
        if let Some(format) = formats
            .iter_mut()
            .find(|format| **format == Format::Screenshot)
        {
            *format = Format::ScreenshotWithOptions(ScreenshotOptions {
                full_page: Some(true),
                ..Default::default()
            });
        } else {
            formats.push(Format::ScreenshotWithOptions(ScreenshotOptions {
                full_page: Some(true),
                ..Default::default()
            }));
        }
    } else if args.get_flag("screenshot") && !formats.contains(&Format::Screenshot) {
        formats.push(Format::Screenshot);
    }
    let actions = if let Some(path) = args.get_one::<std::path::PathBuf>("actions-file") {
        let bytes =
            std::fs::read(path).with_context(|| format!("reading actions {}", path.display()))?;
        Some(serde_json::from_slice(&bytes).context("parsing browser actions")?)
    } else {
        args.get_one::<crate::args::Actions>("actions")
            .map(|actions| actions.0.clone())
    };
    Ok(ScrapeOptions {
        formats: Some(formats),
        only_main_content: one(args, "only-main-content"),
        include_tags: strings(args, "include-tags"),
        exclude_tags: strings(args, "exclude-tags"),
        wait_for: one(args, "wait-for"),
        timeout: one(args, "timeout-ms"),
        mobile: one(args, "mobile"),
        skip_tls_verification: one(args, "skip-tls-verification"),
        remove_base64_images: one(args, "remove-base64-images"),
        block_ads: one(args, "block-ads"),
        proxy: native_one(args, "proxy")?,
        max_age: one(args, "max-age"),
        store_in_cache: one(args, "store-in-cache"),
        lockdown: one(args, "lockdown"),
        redact_pii: one(args, "redact-pii"),
        location: location(args),
        actions,
        ..Default::default()
    })
}

pub fn map(args: &ArgMatches) -> Result<MapOptions> {
    Ok(MapOptions {
        search: one(args, "search"),
        sitemap: native_one(args, "sitemap")?,
        include_subdomains: one(args, "include-subdomains"),
        ignore_query_parameters: one(args, "ignore-query-parameters"),
        limit: one(args, "limit"),
        timeout: one(args, "timeout-ms"),
        location: location(args),
        ..Default::default()
    })
}

pub fn search(args: &ArgMatches) -> Result<SearchOptions> {
    let scrape_options = if args.get_flag("scrape") {
        Some(ScrapeOptions {
            formats: Some(
                native_list(args, "scrape-formats")?.unwrap_or_else(|| vec![Format::Markdown]),
            ),
            only_main_content: one(args, "scrape-only-main-content"),
            ..Default::default()
        })
    } else {
        None
    };
    Ok(SearchOptions {
        limit: one(args, "limit"),
        sources: native_list(args, "sources")?,
        categories: native_list(args, "categories")?,
        tbs: one(args, "tbs"),
        location: one(args, "location"),
        country: one(args, "country"),
        timeout: one(args, "timeout-ms"),
        ignore_invalid_urls: one(args, "ignore-invalid-urls"),
        scrape_options,
        ..Default::default()
    })
}

pub fn agent(args: &ArgMatches) -> Result<AgentOptions> {
    Ok(AgentOptions {
        prompt: args.get_one::<String>("PROMPT").unwrap().clone(),
        urls: strings(args, "urls"),
        schema: schema(args)?,
        max_credits: one(args, "max-credits"),
        strict_constrain_to_urls: one(args, "strict-constrain-to-urls"),
        model: native_one(args, "model")?,
        webhook: one(args, "webhook"),
        ..Default::default()
    })
}
