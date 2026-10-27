use anyhow::{Context, Result};
use cli_command_kit::{Mode, clap::ArgMatches};
use serde::Serialize;
use std::{io::Write, path::Path};

pub fn json(value: &impl Serialize, pretty: bool) -> Result<String> {
    Ok(if pretty {
        serde_json::to_string_pretty(value)?
    } else {
        serde_json::to_string(value)?
    })
}

pub fn emit(args: &ArgMatches, text: &str, output: &mut impl Write) -> Result<()> {
    if let Some(path) = args.get_one::<std::path::PathBuf>("output") {
        let parent = path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let mut file = tempfile::NamedTempFile::new_in(parent)
            .with_context(|| format!("creating output in {}", parent.display()))?;
        file.write_all(text.as_bytes())?;
        file.as_file().sync_all()?;
        file.persist(path)
            .with_context(|| format!("saving output {}", path.display()))?;
    } else {
        writeln!(output, "{text}")?;
    }
    Ok(())
}

pub fn document(
    doc: &firecrawl::Document,
    formats: &[firecrawl::Format],
    as_json: bool,
    pretty: bool,
) -> Result<String> {
    if as_json || formats.len() > 1 {
        return json(doc, pretty);
    }
    let value = serde_json::to_value(doc)?;
    let format = serde_json::to_value(&formats[0])?;
    let name = format
        .as_str()
        .or_else(|| format.get("type").and_then(serde_json::Value::as_str))
        .context("output format has no name")?;
    match value.get(name) {
        Some(serde_json::Value::String(text)) => Ok(text.clone()),
        Some(content) => json(content, pretty),
        None => json(doc, pretty),
    }
}

pub fn map(data: &firecrawl::MapResponse, detailed: bool) -> String {
    if data.links.is_empty() {
        return "No URLs found.".into();
    }
    data.links
        .iter()
        .map(|link| {
            if detailed {
                format!(
                    "{}\n  {}\n  {}",
                    link.url,
                    link.title.as_deref().unwrap_or(""),
                    link.description.as_deref().unwrap_or("")
                )
            } else {
                link.url.clone()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn search(data: &firecrawl::SearchData) -> String {
    let mut lines = Vec::new();
    if let Some(web) = &data.web {
        for item in web {
            match item {
                firecrawl::SearchResultOrDocument::WebResult(result) => {
                    lines.push(format!(
                        "Web: {}\n{}\n{}",
                        result.title.as_deref().unwrap_or(""),
                        result.url,
                        result.description.as_deref().unwrap_or("")
                    ));
                }
                firecrawl::SearchResultOrDocument::Document(doc) => {
                    lines.push(format!(
                        "Web: {}\n{}",
                        doc.metadata
                            .as_ref()
                            .and_then(|meta| meta.source_url.as_deref())
                            .unwrap_or(""),
                        doc.markdown
                            .as_deref()
                            .or(doc.html.as_deref())
                            .unwrap_or("")
                    ));
                }
            }
        }
    }
    if let Some(news) = &data.news {
        for result in news {
            lines.push(format!(
                "News: {}\n{}\n{}",
                result.title.as_deref().unwrap_or(""),
                result.url.as_deref().unwrap_or(""),
                result.snippet.as_deref().unwrap_or("")
            ));
        }
    }
    if let Some(images) = &data.images {
        for result in images {
            lines.push(format!(
                "Image: {}\n{}\n{}",
                result.title.as_deref().unwrap_or(""),
                result.url.as_deref().unwrap_or(""),
                result.image_url.as_deref().unwrap_or("")
            ));
        }
    }
    if lines.is_empty() {
        "No search results found.".into()
    } else {
        lines.join("\n\n")
    }
}

pub fn warning(value: Option<&str>, diagnostics: &mut impl Write, mode: Mode) -> Result<()> {
    if let Some(message) = value.filter(|value| !value.is_empty()) {
        writeln!(
            diagnostics,
            "{} {}",
            if mode == Mode::Agent {
                "WARN:"
            } else {
                "[WARN]"
            },
            cli_command_kit::display_message(message)
        )?;
    }
    Ok(())
}
