use std::io::Read;
use std::time::Duration;

use anyhow::{Context as _, Result, bail};
use serde_json::{Value, json};

use crate::config::{Model, ModelProvider};

const MAX_REPLY_BYTES: u64 = 16 * 1024 * 1024;

pub struct Chat {
    provider: ModelProvider,
    model: String,
    api_key: Option<String>,
    base_url: String,
    agent: ureq::Agent,
}

impl Chat {
    pub fn from_config(cfg: &Model) -> Result<Self> {
        let (default_url, default_key, default_model) = match cfg.provider {
            ModelProvider::Openai => ("https://api.openai.com/v1", "OPENAI_API_KEY", "gpt-5.2"),
            ModelProvider::Openrouter => ("https://openrouter.ai/api/v1", "OPENROUTER_API_KEY", "openai/gpt-5.2"),
            ModelProvider::Compatible => ("", "", ""),
            ModelProvider::Anthropic => bail!("the chat client cannot use the Anthropic provider"),
        };
        let base_url = if cfg.base_url.is_empty() { default_url } else { &cfg.base_url };
        if base_url.is_empty() {
            bail!("model.base_url is required for an OpenAI-compatible provider");
        }
        let base_url = super::validate_base_url(base_url)?;
        let model = if cfg.model.is_empty() { default_model } else { &cfg.model };
        if model.is_empty() {
            bail!("model.model is required for an OpenAI-compatible provider");
        }
        let key_env = if cfg.api_key_env.is_empty() { default_key } else { &cfg.api_key_env };
        let api_key = if key_env.is_empty() {
            None
        } else {
            match std::env::var(key_env) {
                Ok(key) if !key.trim().is_empty() => Some(key),
                _ => bail!("this needs an API key: set {key_env} (or point model.api_key_env at another variable)"),
            }
        };
        let agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .timeout_global(Some(Duration::from_secs(cfg.timeout_secs)))
            .build()
            .into();
        Ok(Self { provider: cfg.provider, model: model.to_string(), api_key, base_url, agent })
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    fn request_body(&self, system: &str, user: &str, schema: &Value, max_tokens: u32) -> Value {
        let mut body = json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": user},
            ],
            "response_format": {
                "type": "json_schema",
                "json_schema": {"name": "bluepencil_result", "strict": true, "schema": schema},
            },
        });
        let token_key = if self.provider == ModelProvider::Openai { "max_completion_tokens" } else { "max_tokens" };
        body[token_key] = json!(max_tokens);
        body
    }

    pub fn complete_json(&self, system: &str, user: &str, schema: &Value, max_tokens: u32) -> Result<Value> {
        let body = self.request_body(system, user, schema, max_tokens);
        let mut req =
            self.agent.post(format!("{}/chat/completions", self.base_url)).header("content-type", "application/json");
        if let Some(key) = &self.api_key {
            req = req.header("authorization", &format!("Bearer {key}"));
        }
        let mut response = req.send_json(&body).context("sending model request")?;
        let status = response.status().as_u16();
        if !(200..300).contains(&status) {
            bail!("model provider returned HTTP {status}; check the provider, model, and API key");
        }
        let mut text = String::new();
        response
            .body_mut()
            .as_reader()
            .take(MAX_REPLY_BYTES + 1)
            .read_to_string(&mut text)
            .context("reading model reply")?;
        if text.len() as u64 > MAX_REPLY_BYTES {
            bail!("model reply exceeded the size limit");
        }
        parse_reply(&serde_json::from_str(&text).context("model provider did not return JSON")?)
    }
}

fn parse_reply(reply: &Value) -> Result<Value> {
    let choice =
        reply["choices"].as_array().and_then(|choices| choices.first()).context("model reply had no choice")?;
    if choice["finish_reason"] == "length" {
        bail!("the model's reply was cut off at the token limit; try fewer files or findings");
    }
    if choice["finish_reason"] == "content_filter" || !choice["message"]["refusal"].is_null() {
        bail!("the model declined this request");
    }
    let content = choice["message"]["content"].as_str().context("model reply had no text content")?;
    serde_json::from_str(content).context("the model's reply was not the JSON it was asked for")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn openai_request_uses_strict_schema() {
        let client = Chat {
            provider: ModelProvider::Openai,
            model: "test-model".into(),
            api_key: None,
            base_url: "https://example.com/v1".into(),
            agent: ureq::Agent::new_with_defaults(),
        };
        let schema = json!({"type": "object", "properties": {}, "additionalProperties": false});
        let body = client.request_body("system", "user", &schema, 128);
        assert_eq!(body["max_completion_tokens"], 128);
        assert_eq!(body["response_format"]["json_schema"]["schema"], schema);
        assert_eq!(body["response_format"]["json_schema"]["strict"], true);
        assert_eq!(body["messages"][0]["role"], "system");
    }

    #[test]
    fn parses_reply_and_rejects_truncation() {
        let reply = json!({"choices": [{"finish_reason": "stop", "message": {"content": "{\"a\":1}"}}]});
        assert_eq!(parse_reply(&reply).unwrap(), json!({"a": 1}));
        let cut = json!({"choices": [{"finish_reason": "length", "message": {"content": "{}"}}]});
        assert!(parse_reply(&cut).unwrap_err().to_string().contains("token limit"));
    }

    #[test]
    fn rejects_non_loopback_http_and_embedded_credentials() {
        let mut cfg = Model { provider: ModelProvider::Compatible, model: "test-model".into(), ..Model::default() };
        cfg.base_url = "http://localhost:1234@remote.example/v1".into();
        assert!(Chat::from_config(&cfg).is_err());
        cfg.base_url = "https://user:password@remote.example/v1".into();
        assert!(Chat::from_config(&cfg).is_err());
        cfg.base_url = "http://127.0.0.1:8080/v1".into();
        assert!(Chat::from_config(&cfg).is_ok());
    }
}
