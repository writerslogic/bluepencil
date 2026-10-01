//! Minimal client for the Claude Messages API over raw HTTP.
//!
//! One call: a system prompt, a user message, and a JSON Schema the reply must satisfy.
//! The API's structured-output mode guarantees the first text block parses against it.
//! Replies are streamed so a long ledger does not hit the HTTP idle timeout.

use std::io::{BufRead, BufReader};
use std::time::Duration;

use anyhow::{Context as _, Result, bail};
use serde_json::{Value, json};

use crate::config;

const API_VERSION: &str = "2023-06-01";
const FALLBACK_BETA: &str = "server-side-fallback-2026-07-01";

pub struct Claude {
    model: String,
    effort: String,
    api_key: String,
    base_url: String,
    fallback: bool,
    agent: ureq::Agent,
}

impl Claude {
    /// Builds a client from the `[model]` config, reading the key from its environment variable.
    pub fn from_config(cfg: &config::Model) -> Result<Self> {
        let api_key = match std::env::var(&cfg.api_key_env) {
            Ok(k) if !k.trim().is_empty() => k,
            _ => {
                bail!("this needs an API key: set {} (or point model.api_key_env at another variable)", cfg.api_key_env)
            }
        };
        let agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .timeout_global(Some(Duration::from_secs(cfg.timeout_secs)))
            .build()
            .into();
        Ok(Self {
            model: cfg.model.clone(),
            effort: cfg.effort.clone(),
            api_key,
            base_url: cfg.base_url.trim_end_matches('/').to_string(),
            fallback: cfg.fallback,
            agent,
        })
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    /// The request body for one structured-output call. Separate from `complete_json` so tests
    /// can check it without a network.
    pub fn request_body(&self, system: &str, user: &str, schema: &Value, max_tokens: u32) -> Value {
        let mut body = json!({
            "model": self.model,
            "max_tokens": max_tokens,
            "stream": true,
            "system": system,
            "messages": [{"role": "user", "content": user}],
            "output_config": {
                "effort": self.effort,
                "format": {"type": "json_schema", "schema": schema},
            },
        });
        if self.fallback {
            body["fallbacks"] = json!("default");
        }
        body
    }

    /// Sends one message and returns the parsed JSON object from the reply.
    pub fn complete_json(&self, system: &str, user: &str, schema: &Value, max_tokens: u32) -> Result<Value> {
        let body = self.request_body(system, user, schema, max_tokens);
        let mut req = self
            .agent
            .post(format!("{}/v1/messages", self.base_url))
            .header("content-type", "application/json")
            .header("accept", "text/event-stream")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", API_VERSION);
        if self.fallback {
            req = req.header("anthropic-beta", FALLBACK_BETA);
        }
        let mut response = req.send_json(&body).context("sending request to the Claude API")?;
        let status = response.status().as_u16();
        if !(200..300).contains(&status) {
            let text = response.body_mut().read_to_string().unwrap_or_default();
            bail!("Claude API returned HTTP {status}: {}", api_error_message(&text));
        }
        let reply = read_stream(BufReader::new(response.body_mut().as_reader()))?;
        parse_reply(&reply)
    }
}

fn api_error_message(text: &str) -> String {
    serde_json::from_str::<Value>(text)
        .ok()
        .and_then(|v| v["error"]["message"].as_str().map(str::to_owned))
        .unwrap_or_else(|| text.chars().take(300).collect())
}

/// Folds a server-sent event stream into the shape of a non-streaming reply: the concatenated
/// text and the final stop reason. Stops on the first `error` event.
pub fn read_stream(reader: impl BufRead) -> Result<Value> {
    let mut text = String::new();
    let mut stop_reason = Value::Null;
    let mut stop_details = Value::Null;
    for line in reader.lines() {
        let line = line.context("reading the Claude API stream")?;
        let Some(data) = line.strip_prefix("data:") else { continue };
        let event: Value = match serde_json::from_str(data.trim()) {
            Ok(v) => v,
            Err(_) => continue,
        };
        match event["type"].as_str() {
            Some("content_block_delta") => {
                if let Some(t) = event["delta"]["text"].as_str() {
                    text.push_str(t);
                }
            }
            Some("message_delta") => {
                if !event["delta"]["stop_reason"].is_null() {
                    stop_reason = event["delta"]["stop_reason"].clone();
                }
                if !event["delta"]["stop_details"].is_null() {
                    stop_details = event["delta"]["stop_details"].clone();
                }
            }
            Some("error") => {
                let msg = event["error"]["message"].as_str().unwrap_or("unknown error");
                bail!("Claude API stream error: {msg}");
            }
            _ => {}
        }
    }
    Ok(json!({
        "stop_reason": stop_reason,
        "stop_details": stop_details,
        "content": [{"type": "text", "text": text}],
    }))
}

/// Extracts the structured JSON from a successful reply, refusing to guess when the model
/// stopped early or declined.
pub fn parse_reply(reply: &Value) -> Result<Value> {
    match reply["stop_reason"].as_str() {
        Some("refusal") => {
            let why = reply["stop_details"]["explanation"].as_str().unwrap_or("no explanation given");
            bail!("the model declined this request: {why}")
        }
        Some("max_tokens") => bail!("the model's reply was cut off at the token limit; try fewer files or findings"),
        _ => {}
    }
    let text = reply["content"]
        .as_array()
        .and_then(|blocks| blocks.iter().find(|b| b["type"] == "text"))
        .and_then(|b| b["text"].as_str())
        .context("the Claude API reply had no text block")?;
    serde_json::from_str(text).context("the model's reply was not the JSON it was asked for")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn client() -> Claude {
        Claude {
            model: "claude-opus-5-5".into(),
            effort: "medium".into(),
            api_key: "test".into(),
            base_url: "http://127.0.0.1:1".into(),
            fallback: true,
            agent: ureq::Agent::new_with_defaults(),
        }
    }

    #[test]
    fn request_uses_structured_output_effort_and_streaming() {
        let schema = json!({"type": "object", "properties": {}, "additionalProperties": false});
        let body = client().request_body("sys", "hello", &schema, 16_000);
        assert_eq!(body["model"], "claude-opus-5-5");
        assert_eq!(body["stream"], true);
        assert_eq!(body["max_tokens"], 16_000);
        assert_eq!(body["output_config"]["effort"], "medium");
        assert_eq!(body["output_config"]["format"]["type"], "json_schema");
        assert_eq!(body["output_config"]["format"]["schema"], schema);
        assert_eq!(body["messages"][0]["content"], "hello");
        assert_eq!(body["fallbacks"], "default");
    }

    #[test]
    fn stream_concatenates_text_deltas_and_keeps_the_stop_reason() {
        let sse = "event: message_start\ndata: {\"type\":\"message_start\"}\n\n\
                   data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"{\\\"a\\\":\"}}\n\n\
                   data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\" 1}\"}}\n\n\
                   data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"}}\n\n\
                   data: {\"type\":\"message_stop\"}\n";
        let reply = read_stream(sse.as_bytes()).unwrap();
        assert_eq!(parse_reply(&reply).unwrap(), json!({"a": 1}));
    }

    #[test]
    fn stream_error_event_is_an_error() {
        let sse = "data: {\"type\":\"error\",\"error\":{\"type\":\"overloaded_error\",\"message\":\"Overloaded\"}}\n";
        assert!(read_stream(sse.as_bytes()).unwrap_err().to_string().contains("Overloaded"));
    }

    #[test]
    fn refusal_and_truncation_are_errors_not_empty_results() {
        let refused = json!({
            "stop_reason": "refusal",
            "stop_details": {"type": "refusal", "explanation": "policy"},
            "content": [],
        });
        assert!(parse_reply(&refused).unwrap_err().to_string().contains("policy"));
        let cut = json!({"stop_reason": "max_tokens", "content": [{"type": "text", "text": "{\"notes\": ["}]});
        assert!(parse_reply(&cut).unwrap_err().to_string().contains("token limit"));
    }
}
