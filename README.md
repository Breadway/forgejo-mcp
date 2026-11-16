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
`create_issue`, `edit_issue`, `comment_issue`, `list_pulls`, `get_pull`,
`create_pull`, `edit_pull`, `merge_pull`, `list_pull_files`, `list_branches`,
`get_file_contents`, `list_commits`, `get_commit_status`, `list_labels`,
`list_milestones`, `whoami`, `list_my_orgs`.

List endpoints (`list_repos`, `search_repos`, `list_issues`, `list_pulls`,
`list_pull_files`, `list_branches`, `list_commits`, `list_labels`,
`list_milestones`) take optional `page`/`limit` args.

`edit_issue`/`edit_pull` are metadata-only PATCH (title/body/state/base) —
reversible, no merge or delete capability.

`merge_pull` actually merges (`style`: merge/rebase/rebase-merge/squash/
fast-forward-only, default `merge`; optional `title`/`message`;
`delete_branch` to remove the head branch after merging) — irreversible via
this server. Not implemented (higher-risk, harder to reverse): repo
create/delete, file write/delete, release/tag management.

## Claude Code

```
claude mcp add forgejo /home/breadway/Projects/forgejo-mcp/target/release/forgejo-mcp
```
