use serde_json::Value;

/// Minimal client for the Forgejo/Gitea-compatible REST API (v1).
pub struct Client {
    base_url: String,
    token: String,
    agent: ureq::Agent,
}

impl Client {
    pub fn new(base_url: String, token: String) -> Self {
        let base_url = base_url.trim_end_matches('/').to_string();
        Client {
            base_url,
            token,
            agent: ureq::AgentBuilder::new().build(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}/api/v1{}", self.base_url, path)
    }

    pub fn get(&self, path: &str) -> Result<Value, String> {
        let req = self
            .agent
            .get(&self.url(path))
            .set("Authorization", &format!("token {}", self.token));
        Self::handle(req.call())
    }

    pub fn post(&self, path: &str, body: &Value) -> Result<Value, String> {
        let req = self
            .agent
            .post(&self.url(path))
            .set("Authorization", &format!("token {}", self.token));
        Self::handle(req.send_json(body.clone()))
    }

    pub fn patch(&self, path: &str, body: &Value) -> Result<Value, String> {
        let req = self
            .agent
            .request("PATCH", &self.url(path))
            .set("Authorization", &format!("token {}", self.token));
        Self::handle(req.send_json(body.clone()))
    }

    fn handle(result: Result<ureq::Response, ureq::Error>) -> Result<Value, String> {
        match result {
            Ok(resp) => {
                if resp.status() == 204 {
                    return Ok(Value::Null);
                }
                resp.into_json::<Value>()
                    .map_err(|e| format!("failed to parse response JSON: {e}"))
            }
            Err(ureq::Error::Status(code, resp)) => {
                let body = resp.into_string().unwrap_or_default();
                Err(format!("HTTP {code}: {body}"))
            }
            Err(ureq::Error::Transport(t)) => Err(format!("transport error: {t}")),
        }
    }
}

/// Percent-encode a single path segment (owner, repo, branch names, file paths, etc).
pub fn encode_segment(segment: &str) -> String {
    let mut out = String::with_capacity(segment.len());
    for byte in segment.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char)
            }
            _ => out.push_str(&format!("%{:02X}", byte)),
        }
    }
    out
}
