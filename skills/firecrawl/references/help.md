---
created_on: 2026-10-14 17:18
last_modified: 2026-10-14 17:38
status: current
---

## Completion scripts

Each shell command prints its native completion script in both modes without API access. It accepts global flags and `--help`/`-h`, with no command-specific flags or positional arguments:

- `firecrawl completion bash`
- `firecrawl completion zsh`
- `firecrawl completion fish`
- `firecrawl completion powershell`
- `firecrawl completion elvish`

Save/load the script through the selected shell's completion facilities.

## Generated help queries

These commands print the selected help page without API access. `help` trailing tokens select a help target; unknown targets fail validation. Agent help begins with the skill-reading alert. Every command also accepts `--help`/`-h`.

- `firecrawl help`
- `firecrawl help help`
- `firecrawl help page`
- `firecrawl help page scrape`
- `firecrawl help site`
- `firecrawl help site map`
- `firecrawl help web`
- `firecrawl help web search`
- `firecrawl help agent`
- `firecrawl help agent create`
- `firecrawl help agent status`
- `firecrawl help agent cancel`
- `firecrawl help skill`
- `firecrawl help completion`
- `firecrawl help completion bash`
- `firecrawl help completion zsh`
- `firecrawl help completion fish`
- `firecrawl help completion powershell`
- `firecrawl help completion elvish`
- `firecrawl page help`
- `firecrawl page help help`
- `firecrawl page help scrape`
- `firecrawl site help`
- `firecrawl site help help`
- `firecrawl site help map`
- `firecrawl web help`
- `firecrawl web help help`
- `firecrawl web help search`
- `firecrawl agent help`
- `firecrawl agent help help`
- `firecrawl agent help create`
- `firecrawl agent help status`
- `firecrawl agent help cancel`
- `firecrawl completion help`
- `firecrawl completion help help`
- `firecrawl completion help bash`
- `firecrawl completion help zsh`
- `firecrawl completion help fish`
- `firecrawl completion help powershell`
- `firecrawl completion help elvish`
