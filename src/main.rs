mod forgejo;
mod tools;

use std::io::{self, BufRead, Write};

use serde_json::{json, Value};

use forgejo::Client;

const PROTOCOL_VERSION: &str = "2025-06-18";
const SERVER_NAME: &str = "forgejo-mcp";
const SERVER_VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let base_url = std::env::var("FORGEJO_URL").unwrap_or_else(|_| "https://git.breadway.dev".into());
    let token = std::env::var("FORGEJO_TOKEN").unwrap_or_else(|_| {
        let path = shellexpand_home("~/.config/forgejo/token");
        std::fs::read_to_string(&path)
            .unwrap_or_else(|e| {
                eprintln!("forgejo-mcp: no FORGEJO_TOKEN set and failed to read {path}: {e}");
                std::process::exit(1);
            })
            .trim()
            .to_string()
    });
    let client = Client::new(base_url, token);

    let stdin = io::stdin();
    let mut stdout = io::stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("forgejo-mcp: stdin read error: {e}");
                break;
            }
        };
        if line.trim().is_empty() {
            continue;
        }
        let request: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("forgejo-mcp: invalid JSON on stdin: {e}");
                continue;
            }
        };

        if let Some(response) = handle_message(&client, &request) {
            let text = serde_json::to_string(&response).expect("response is always serializable");
            if writeln!(stdout, "{text}").is_err() || stdout.flush().is_err() {
                break;
            }
        }
    }
}

fn handle_message(client: &Client, request: &Value) -> Option<Value> {
    let method = request.get("method").and_then(Value::as_str)?;
    let id = request.get("id").cloned();

    // Notifications carry no id and expect no response.
    if id.is_none() {
        return None;
    }

    let result = match method {
        "initialize" => Ok(json!({
            "protocolVersion": PROTOCOL_VERSION,
            "capabilities": { "tools": {} },
            "serverInfo": { "name": SERVER_NAME, "version": SERVER_VERSION }
        })),
        "ping" => Ok(json!({})),
        "tools/list" => Ok(json!({
            "tools": tools::list().into_iter().map(|t| json!({
                "name": t.name,
                "description": t.description,
                "inputSchema": t.schema,
            })).collect::<Vec<_>>()
        })),
        "tools/call" => Ok(handle_tool_call(client, request)),
        _ => Err((-32601, format!("method not found: {method}"))),
    };

    Some(match result {
        Ok(result) => json!({ "jsonrpc": "2.0", "id": id, "result": result }),
        Err((code, message)) => json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": code, "message": message }
        }),
    })
}

fn handle_tool_call(client: &Client, request: &Value) -> Value {
    let params = request.get("params").cloned().unwrap_or(Value::Null);
    let name = params.get("name").and_then(Value::as_str).unwrap_or_default();
    let args = params.get("arguments").cloned().unwrap_or_else(|| json!({}));

    match tools::call(client, name, &args) {
        Ok(value) => json!({
            "content": [{ "type": "text", "text": serde_json::to_string_pretty(&value).unwrap_or_default() }],
            "isError": false
        }),
        Err(message) => json!({
            "content": [{ "type": "text", "text": message }],
            "isError": true
        }),
    }
}

fn shellexpand_home(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{home}/{rest}");
        }
    }
    path.to_string()
}
