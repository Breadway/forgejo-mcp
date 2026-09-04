# forgejo-mcp

A minimal MCP server for Forgejo (and Gitea-compatible) instances. No async
runtime, no MCP SDK — hand-rolled JSON-RPC 2.0 over stdio, with `ureq` as the
only HTTP dependency. Builds to a single static-ish binary.

## Config

Environment variables (both optional):

- `FORGEJO_URL` — base URL of the instance. Defaults to `https://git.breadway.dev`.
- `FORGEJO_TOKEN` — API token. Defaults to reading `~/.config/forgejo/token`.

## Build

```
cargo build --release
```

Binary at `target/release/forgejo-mcp`.

## Tools

`list_repos`, `search_repos`, `get_repo`, `list_issues`, `get_issue`,
`create_issue`, `comment_issue`, `list_pulls`, `get_pull`, `create_pull`,
`list_branches`, `get_file_contents`, `list_commits`.

## Claude Code

```
claude mcp add forgejo /home/breadway/Projects/forgejo-mcp/target/release/forgejo-mcp
```
