use serde_json::{json, Value};

use crate::forgejo::{encode_segment, Client};

pub struct Tool {
    pub name: &'static str,
    pub description: &'static str,
    pub schema: Value,
}

const PAGE: (&str, &str, &str) = ("page", "integer", "Page number (default 1)");
const LIMIT: (&str, &str, &str) = ("limit", "integer", "Results per page (default 20)");

pub fn list() -> Vec<Tool> {
    vec![
        Tool {
            name: "list_repos",
            description: "List repositories owned by a user or organization",
            schema: with_fields(
                &[("owner", "string!", "Username or org name"), PAGE, LIMIT],
                &[],
            ),
        },
        Tool {
            name: "search_repos",
            description: "Search repositories by keyword across the instance",
            schema: with_fields(&[("query", "string!", "Search keywords"), PAGE, LIMIT], &[]),
        },
        Tool {
            name: "get_repo",
            description: "Get details of a single repository",
            schema: owner_repo_schema(&[]),
        },
        Tool {
            name: "list_issues",
            description: "List issues in a repository",
            schema: owner_repo_schema(&[
                ("state", "string", "open, closed, or all (default open)"),
                PAGE,
                LIMIT,
            ]),
        },
        Tool {
            name: "get_issue",
            description: "Get a single issue by index",
            schema: owner_repo_schema(&[("index", "integer!", "Issue number")]),
        },
        Tool {
            name: "create_issue",
            description: "Create a new issue in a repository",
            schema: owner_repo_schema(&[
                ("title", "string!", "Issue title"),
                ("body", "string", "Issue body/description"),
            ]),
        },
        Tool {
            name: "edit_issue",
            description: "Edit an issue's title, body, or open/closed state",
            schema: owner_repo_schema(&[
                ("index", "integer!", "Issue number"),
                ("title", "string", "New title"),
                ("body", "string", "New body"),
                ("state", "string", "open or closed"),
            ]),
        },
        Tool {
            name: "comment_issue",
            description: "Add a comment to an issue or pull request",
            schema: owner_repo_schema(&[
                ("index", "integer!", "Issue or PR number"),
                ("body", "string!", "Comment text"),
            ]),
        },
        Tool {
            name: "list_pulls",
            description: "List pull requests in a repository",
            schema: owner_repo_schema(&[
                ("state", "string", "open, closed, or all (default open)"),
                PAGE,
                LIMIT,
            ]),
        },
        Tool {
            name: "get_pull",
            description: "Get a single pull request by index",
            schema: owner_repo_schema(&[("index", "integer!", "PR number")]),
        },
        Tool {
            name: "create_pull",
            description: "Open a new pull request",
            schema: owner_repo_schema(&[
                ("title", "string!", "PR title"),
                ("head", "string!", "Source branch"),
                ("base", "string!", "Target branch"),
                ("body", "string", "PR description"),
            ]),
        },
        Tool {
            name: "edit_pull",
            description: "Edit a pull request's title, body, target branch, or open/closed state (does not merge)",
            schema: owner_repo_schema(&[
                ("index", "integer!", "PR number"),
                ("title", "string", "New title"),
                ("body", "string", "New body"),
                ("base", "string", "New target branch"),
                ("state", "string", "open or closed"),
            ]),
        },
        Tool {
            name: "merge_pull",
            description: "Merge a pull request",
            schema: owner_repo_schema(&[
                ("index", "integer!", "PR number"),
                (
                    "style",
                    "string",
                    "Merge style: merge, rebase, rebase-merge, squash, or fast-forward-only (default merge)",
                ),
                ("title", "string", "Merge commit title (optional)"),
                ("message", "string", "Merge commit message (optional)"),
                ("delete_branch", "boolean", "Delete the head branch after merging (default false)"),
            ]),
        },
        Tool {
            name: "list_pull_files",
            description: "List files changed by a pull request",
            schema: owner_repo_schema(&[("index", "integer!", "PR number"), PAGE, LIMIT]),
        },
        Tool {
            name: "list_branches",
            description: "List branches in a repository",
            schema: owner_repo_schema(&[PAGE, LIMIT]),
        },
        Tool {
            name: "get_file_contents",
            description: "Get the contents of a file in a repository (base64-decoded if text)",
            schema: owner_repo_schema(&[
                ("path", "string!", "File path within the repo"),
                ("ref", "string", "Branch, tag, or commit SHA (default: repo default branch)"),
            ]),
        },
        Tool {
            name: "list_commits",
            description: "List recent commits in a repository",
            schema: owner_repo_schema(&[
                ("ref", "string", "Branch, tag, or commit SHA"),
                PAGE,
                LIMIT,
            ]),
        },
        Tool {
            name: "get_commit_status",
            description: "Get the combined CI/commit status for a branch, tag, or commit SHA",
            schema: owner_repo_schema(&[("ref", "string!", "Branch, tag, or commit SHA")]),
        },
        Tool {
            name: "list_labels",
            description: "List labels defined in a repository",
            schema: owner_repo_schema(&[PAGE, LIMIT]),
        },
        Tool {
            name: "list_milestones",
            description: "List milestones in a repository",
            schema: owner_repo_schema(&[
                ("state", "string", "open, closed, or all (default open)"),
                PAGE,
                LIMIT,
            ]),
        },
        Tool {
            name: "whoami",
            description: "Get the authenticated user for the configured token",
            schema: json!({ "type": "object", "properties": {} }),
        },
        Tool {
            name: "list_my_orgs",
            description: "List organizations the authenticated user belongs to",
            schema: json!({ "type": "object", "properties": {} }),
        },
    ]
}

/// Builds a JSON Schema object from a field spec list.
/// Field spec: (name, "type" or "type!" for required, description).
fn with_fields(base: &[(&str, &str, &str)], extra: &[(&str, &str, &str)]) -> Value {
    let mut properties = serde_json::Map::new();
    let mut required = vec![];
    for (name, ty, desc) in base.iter().chain(extra.iter()) {
        let (ty, is_required) = match ty.strip_suffix('!') {
            Some(t) => (t, true),
            None => (*ty, false),
        };
        properties.insert(name.to_string(), json!({ "type": ty, "description": desc }));
        if is_required {
            required.push(name.to_string());
        }
    }
    json!({
        "type": "object",
        "properties": properties,
        "required": required
    })
}

/// Same as `with_fields` but always prepends the required owner/repo pair.
fn owner_repo_schema(extra: &[(&str, &str, &str)]) -> Value {
    let base: &[(&str, &str, &str)] = &[
        ("owner", "string!", "Repository owner (user or org)"),
        ("repo", "string!", "Repository name"),
    ];
    with_fields(base, extra)
}

fn arg_str<'a>(args: &'a Value, key: &str) -> Result<&'a str, String> {
    args.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing required string argument '{key}'"))
}

fn arg_str_opt<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(Value::as_str)
}

fn arg_int(args: &Value, key: &str) -> Result<i64, String> {
    args.get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| format!("missing required integer argument '{key}'"))
}

fn arg_int_opt(args: &Value, key: &str) -> Option<i64> {
    args.get(key).and_then(Value::as_i64)
}

fn encode_path(path: &str) -> String {
    path.split('/').map(encode_segment).collect::<Vec<_>>().join("/")
}

/// Builds a `page=`/`limit=` query fragment (without leading `?`/`&`) from
/// whichever of the two args are present.
fn paging_query(args: &Value) -> String {
    let mut parts = vec![];
    if let Some(p) = arg_int_opt(args, "page") {
        parts.push(format!("page={p}"));
    }
    if let Some(l) = arg_int_opt(args, "limit") {
        parts.push(format!("limit={l}"));
    }
    parts.join("&")
}

fn append_query(base: String, extra: &str) -> String {
    if extra.is_empty() {
        return base;
    }
    if base.contains('?') {
        format!("{base}&{extra}")
    } else {
        format!("{base}?{extra}")
    }
}

/// Builds a PATCH body containing only the string fields present in `args`.
fn patch_body(args: &Value, fields: &[&str]) -> Value {
    let mut map = serde_json::Map::new();
    for f in fields {
        if let Some(v) = arg_str_opt(args, f) {
            map.insert(f.to_string(), json!(v));
        }
    }
    Value::Object(map)
}

pub fn call(client: &Client, name: &str, args: &Value) -> Result<Value, String> {
    match name {
        "list_repos" => {
            let owner = arg_str(args, "owner")?;
            let url = append_query(
                format!("/users/{}/repos", encode_segment(owner)),
                &paging_query(args),
            );
            client.get(&url)
        }
        "search_repos" => {
            let query = arg_str(args, "query")?;
            let url = append_query(
                format!("/repos/search?q={}", encode_segment(query)),
                &paging_query(args),
            );
            client.get(&url)
        }
        "get_repo" => {
            let (owner, repo) = owner_repo(args)?;
            client.get(&format!("/repos/{}/{}", encode_segment(owner), encode_segment(repo)))
        }
        "list_issues" => {
            let (owner, repo) = owner_repo(args)?;
            let state = arg_str_opt(args, "state").unwrap_or("open");
            let url = append_query(
                format!(
                    "/repos/{}/{}/issues?state={}&type=issues",
                    encode_segment(owner),
                    encode_segment(repo),
                    encode_segment(state)
                ),
                &paging_query(args),
            );
            client.get(&url)
        }
        "get_issue" => {
            let (owner, repo) = owner_repo(args)?;
            let index = arg_int(args, "index")?;
            client.get(&format!(
                "/repos/{}/{}/issues/{}",
                encode_segment(owner),
                encode_segment(repo),
                index
            ))
        }
        "create_issue" => {
            let (owner, repo) = owner_repo(args)?;
            let title = arg_str(args, "title")?;
            let body = arg_str_opt(args, "body").unwrap_or("");
            client.post(
                &format!("/repos/{}/{}/issues", encode_segment(owner), encode_segment(repo)),
                &json!({ "title": title, "body": body }),
            )
        }
        "edit_issue" => {
            let (owner, repo) = owner_repo(args)?;
            let index = arg_int(args, "index")?;
            let body = patch_body(args, &["title", "body", "state"]);
            if body.as_object().map(|m| m.is_empty()).unwrap_or(true) {
                return Err("provide at least one of title, body, or state".into());
            }
            client.patch(
                &format!(
                    "/repos/{}/{}/issues/{}",
                    encode_segment(owner),
                    encode_segment(repo),
                    index
                ),
                &body,
            )
        }
        "comment_issue" => {
            let (owner, repo) = owner_repo(args)?;
            let index = arg_int(args, "index")?;
            let body = arg_str(args, "body")?;
            client.post(
                &format!(
                    "/repos/{}/{}/issues/{}/comments",
                    encode_segment(owner),
                    encode_segment(repo),
                    index
                ),
                &json!({ "body": body }),
            )
        }
        "list_pulls" => {
            let (owner, repo) = owner_repo(args)?;
            let state = arg_str_opt(args, "state").unwrap_or("open");
            let url = append_query(
                format!(
                    "/repos/{}/{}/pulls?state={}",
                    encode_segment(owner),
                    encode_segment(repo),
                    encode_segment(state)
                ),
                &paging_query(args),
            );
            client.get(&url)
        }
        "get_pull" => {
            let (owner, repo) = owner_repo(args)?;
            let index = arg_int(args, "index")?;
            client.get(&format!(
                "/repos/{}/{}/pulls/{}",
                encode_segment(owner),
                encode_segment(repo),
                index
            ))
        }
        "create_pull" => {
            let (owner, repo) = owner_repo(args)?;
            let title = arg_str(args, "title")?;
            let head = arg_str(args, "head")?;
            let base = arg_str(args, "base")?;
            let body = arg_str_opt(args, "body").unwrap_or("");
            client.post(
                &format!("/repos/{}/{}/pulls", encode_segment(owner), encode_segment(repo)),
                &json!({ "title": title, "head": head, "base": base, "body": body }),
            )
        }
        "edit_pull" => {
            let (owner, repo) = owner_repo(args)?;
            let index = arg_int(args, "index")?;
            let body = patch_body(args, &["title", "body", "base", "state"]);
            if body.as_object().map(|m| m.is_empty()).unwrap_or(true) {
                return Err("provide at least one of title, body, base, or state".into());
            }
            client.patch(
                &format!(
                    "/repos/{}/{}/pulls/{}",
                    encode_segment(owner),
                    encode_segment(repo),
                    index
                ),
                &body,
            )
        }
        "merge_pull" => {
            let (owner, repo) = owner_repo(args)?;
            let index = arg_int(args, "index")?;
            let style = arg_str_opt(args, "style").unwrap_or("merge");
            let mut body = serde_json::Map::new();
            body.insert("Do".to_string(), json!(style));
            if let Some(t) = arg_str_opt(args, "title") {
                body.insert("MergeTitleField".to_string(), json!(t));
            }
            if let Some(m) = arg_str_opt(args, "message") {
                body.insert("MergeMessageField".to_string(), json!(m));
            }
            if let Some(d) = args.get("delete_branch").and_then(Value::as_bool) {
                body.insert("delete_branch_after_merge".to_string(), json!(d));
            }
            client.post(
                &format!(
                    "/repos/{}/{}/pulls/{}/merge",
                    encode_segment(owner),
                    encode_segment(repo),
                    index
                ),
                &Value::Object(body),
            )
        }
        "list_pull_files" => {
            let (owner, repo) = owner_repo(args)?;
            let index = arg_int(args, "index")?;
            let url = append_query(
                format!(
                    "/repos/{}/{}/pulls/{}/files",
                    encode_segment(owner),
                    encode_segment(repo),
                    index
                ),
                &paging_query(args),
            );
            client.get(&url)
        }
        "list_branches" => {
            let (owner, repo) = owner_repo(args)?;
            let url = append_query(
                format!("/repos/{}/{}/branches", encode_segment(owner), encode_segment(repo)),
                &paging_query(args),
            );
            client.get(&url)
        }
        "get_file_contents" => {
            let (owner, repo) = owner_repo(args)?;
            let path = arg_str(args, "path")?;
            let mut url = format!(
                "/repos/{}/{}/contents/{}",
                encode_segment(owner),
                encode_segment(repo),
                encode_path(path)
            );
            if let Some(reference) = arg_str_opt(args, "ref") {
                url.push_str(&format!("?ref={}", encode_segment(reference)));
            }
            client.get(&url)
        }
        "list_commits" => {
            let (owner, repo) = owner_repo(args)?;
            let mut query = paging_query(args);
            if let Some(reference) = arg_str_opt(args, "ref") {
                if !query.is_empty() {
                    query.push('&');
                }
                query.push_str(&format!("sha={}", encode_segment(reference)));
            }
            let url = append_query(
                format!("/repos/{}/{}/commits", encode_segment(owner), encode_segment(repo)),
                &query,
            );
            client.get(&url)
        }
        "get_commit_status" => {
            let (owner, repo) = owner_repo(args)?;
            let reference = arg_str(args, "ref")?;
            client.get(&format!(
                "/repos/{}/{}/commits/{}/status",
                encode_segment(owner),
                encode_segment(repo),
                encode_segment(reference)
            ))
        }
        "list_labels" => {
            let (owner, repo) = owner_repo(args)?;
            let url = append_query(
                format!("/repos/{}/{}/labels", encode_segment(owner), encode_segment(repo)),
                &paging_query(args),
            );
            client.get(&url)
        }
        "list_milestones" => {
            let (owner, repo) = owner_repo(args)?;
            let state = arg_str_opt(args, "state").unwrap_or("open");
            let url = append_query(
                format!(
                    "/repos/{}/{}/milestones?state={}",
                    encode_segment(owner),
                    encode_segment(repo),
                    encode_segment(state)
                ),
                &paging_query(args),
            );
            client.get(&url)
        }
        "whoami" => client.get("/user"),
        "list_my_orgs" => client.get("/user/orgs"),
        other => Err(format!("unknown tool '{other}'")),
    }
}

fn owner_repo(args: &Value) -> Result<(&str, &str), String> {
    Ok((arg_str(args, "owner")?, arg_str(args, "repo")?))
}
