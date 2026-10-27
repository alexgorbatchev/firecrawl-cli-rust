use cli_command_kit::Mode;
use std::{ffi::OsString, fs};
use tokio_util::sync::CancellationToken;

#[tokio::test]
async fn guide_matches_the_native_interface_and_every_help_path_has_the_agent_alert() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut guide = fs::read_to_string(root.join("skills/firecrawl/SKILL.md")).unwrap();
    assert!(guide.lines().any(|line| line == "name: firecrawl"));
    for topic in ["pages", "map", "search", "agent", "help"] {
        guide.push_str(
            &fs::read_to_string(root.join(format!("skills/firecrawl/references/{topic}.md")))
                .unwrap(),
        );
    }
    let mut command = cli_command_kit::setup(
        firecrawl_cli::command(),
        Mode::Agent,
        &firecrawl_cli::help_options(),
    )
    .unwrap();
    let missing =
        cli_command_kit::skill_omissions(&command, &firecrawl_cli::help_options(), &guide);
    assert!(missing.is_empty(), "{missing:?}");
    command.build();
    fn visit(
        command: &cli_command_kit::clap::Command,
        path: Vec<String>,
        queries: &mut Vec<Vec<String>>,
    ) {
        let mut query = path.clone();
        if !query.iter().any(|part| part == "help") {
            query.push("--help".into());
        }
        queries.push(query);
        for child in command.get_subcommands() {
            let mut next = path.clone();
            next.push(child.get_name().into());
            visit(child, next, queries);
        }
    }
    let mut queries = Vec::new();
    visit(&command, vec!["firecrawl".into()], &mut queries);
    for query in queries {
        let mut output = Vec::new();
        let mut errors = Vec::new();
        let code = firecrawl_cli::entry(
            query.iter().map(OsString::from),
            &mut output,
            &mut errors,
            Mode::Agent,
            &CancellationToken::new(),
        )
        .await;
        assert_eq!(code, 0, "{query:?}: {}", String::from_utf8_lossy(&errors));
        assert!(
            output.starts_with(
                b"ALERT: Agents must read `AGENT=1 firecrawl skill` before using this tool.\n"
            ),
            "{query:?}"
        );
    }
    let human = cli_command_kit::setup(
        firecrawl_cli::command(),
        Mode::Human,
        &firecrawl_cli::help_options(),
    )
    .unwrap()
    .try_get_matches_from(["firecrawl", "--help"])
    .unwrap_err()
    .to_string();
    assert!(human.contains("├─") && human.contains("╰─"));
    let mut outputs = Vec::new();
    for mode in [Mode::Human, Mode::Agent] {
        let mut output = Vec::new();
        let mut errors = Vec::new();
        assert_eq!(
            firecrawl_cli::entry(
                ["firecrawl", "skill"].map(OsString::from),
                &mut output,
                &mut errors,
                mode,
                &CancellationToken::new()
            )
            .await,
            0
        );
        assert!(errors.is_empty());
        outputs.push(output);
    }
    assert_eq!(outputs[0], outputs[1]);
    assert!(
        String::from_utf8(outputs.remove(0)).unwrap().contains(
            "https://github.com/alexgorbatchev/firecrawl-cli/tree/main/skills/firecrawl\n"
        )
    );
    for shell in ["bash", "zsh", "fish", "powershell", "elvish"] {
        let mut output = Vec::new();
        let mut errors = Vec::new();
        assert_eq!(
            firecrawl_cli::entry(
                ["firecrawl", "completion", shell].map(OsString::from),
                &mut output,
                &mut errors,
                Mode::Agent,
                &CancellationToken::new()
            )
            .await,
            0
        );
        assert!(!output.is_empty());
        assert!(errors.is_empty());
    }
}
