use serde_json::{json, Value};

use crate::forgejo::{encode_segment, Client};

pub struct Tool {
    pub name: &'static str,
    pub description: &'static str,
    pub schema: Value,
}

pub fn list() -> Vec<Tool> {
    vec![
        Tool {
            name: "list_repos",
            description: "List repositories owned by a user or organization",
            schema: json!({
                "type": "object",
                "properties": {
                    "owner": { "type": "string", "description": "Username or org name" }
                },
                "required": ["owner"]
            }),
        },
        Tool {
            name: "search_repos",
            description: "Search repositories by keyword across the instance",
            schema: json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" },
                    "limit": { "type": "integer", "description": "Max results (default 20)" }
                },
                "required": ["query"]
            }),
        },
        Tool {
            name: "get_repo",
            description: "Get details of a single repository",
            schema: owner_repo_schema(&[]),
        },
        Tool {
            name: "list_issues",
            description: "List issues in a repository",
            schema: owner_repo_schema(&[("state", "string", "open, closed, or all (default open)")]),
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
            schema: owner_repo_schema(&[("state", "string", "open, closed, or all (default open)")]),
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
            name: "list_branches",
            description: "List branches in a repository",
            schema: owner_repo_schema(&[]),
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
                ("limit", "integer", "Max results (default 20)"),
            ]),
        },
    ]
}

/// Builds a JSON Schema object with owner/repo required, plus extra fields.
/// Field spec: (name, "type" or "type!" for required, description).
fn owner_repo_schema(extra: &[(&str, &str, &str)]) -> Value {
    let mut properties = json!({
        "owner": { "type": "string", "description": "Repository owner (user or org)" },
        "repo": { "type": "string", "description": "Repository name" }
    });
    let mut required = vec!["owner".to_string(), "repo".to_string()];
    let props = properties.as_object_mut().unwrap();
    for (name, ty, desc) in extra {
        let (ty, is_required) = match ty.strip_suffix('!') {
            Some(t) => (t, true),
            None => (*ty, false),
        };
        props.insert(name.to_string(), json!({ "type": ty, "description": desc }));
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

pub fn call(client: &Client, name: &str, args: &Value) -> Result<Value, String> {
    match name {
        "list_repos" => {
            let owner = arg_str(args, "owner")?;
            client.get(&format!("/users/{}/repos", encode_segment(owner)))
        }
        "search_repos" => {
            let query = arg_str(args, "query")?;
            let limit = arg_int_opt(args, "limit").unwrap_or(20);
            client.get(&format!(
                "/repos/search?q={}&limit={}",
                encode_segment(query),
                limit
            ))
        }
        "get_repo" => {
            let (owner, repo) = owner_repo(args)?;
            client.get(&format!("/repos/{}/{}", encode_segment(owner), encode_segment(repo)))
        }
        "list_issues" => {
            let (owner, repo) = owner_repo(args)?;
            let state = arg_str_opt(args, "state").unwrap_or("open");
            client.get(&format!(
                "/repos/{}/{}/issues?state={}&type=issues",
                encode_segment(owner),
                encode_segment(repo),
                encode_segment(state)
            ))
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
            client.get(&format!(
                "/repos/{}/{}/pulls?state={}",
                encode_segment(owner),
                encode_segment(repo),
                encode_segment(state)
            ))
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
        "list_branches" => {
            let (owner, repo) = owner_repo(args)?;
            client.get(&format!(
                "/repos/{}/{}/branches",
                encode_segment(owner),
                encode_segment(repo)
            ))
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
            let mut query = vec![format!("limit={}", arg_int_opt(args, "limit").unwrap_or(20))];
            if let Some(reference) = arg_str_opt(args, "ref") {
                query.push(format!("sha={}", encode_segment(reference)));
            }
            client.get(&format!(
                "/repos/{}/{}/commits?{}",
                encode_segment(owner),
                encode_segment(repo),
                query.join("&")
            ))
        }
        other => Err(format!("unknown tool '{other}'")),
    }
}

fn owner_repo(args: &Value) -> Result<(&str, &str), String> {
    Ok((arg_str(args, "owner")?, arg_str(args, "repo")?))
}
