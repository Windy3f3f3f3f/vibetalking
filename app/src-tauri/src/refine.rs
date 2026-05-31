use crate::settings::Settings;
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::time::Duration;

pub async fn refine(raw: &str, settings: &Settings) -> Result<String> {
    let api_key = refine_api_key(settings)?;
    let endpoint = chat_endpoint(settings);
    let model = non_empty_or(&settings.refine_model, crate::config::REFINE_MODEL);
    let system = non_empty_or(&settings.refine_prompt, crate::config::REFINE_PROMPT);

    let payload = json!({
        "model": model,
        "messages": [
            { "role": "system", "content": system },
            { "role": "user", "content": raw },
        ],
        "temperature": 0.2,
        "stream": false,
    });

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()?;
    let resp = client
        .post(&endpoint)
        .bearer_auth(&api_key)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!(
            "refine request failed status={} body={}",
            status,
            body.chars().take(400).collect::<String>()
        ));
    }
    let data: Value = serde_json::from_str(&body)
        .map_err(|e| anyhow!("invalid refine response: {}; body={}", e, body.chars().take(400).collect::<String>()))?;
    let text = data
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if text.is_empty() {
        return Err(anyhow!("empty refine result"));
    }
    Ok(strip_wrapping_fences(&text))
}

fn chat_endpoint(settings: &Settings) -> String {
    let base = non_empty_or(&settings.refine_base_url, crate::config::REFINE_BASE_URL);
    format!("{}/chat/completions", base.trim_end_matches('/'))
}

fn refine_api_key(settings: &Settings) -> Result<String> {
    let key = settings.refine_api_key.trim();
    if !key.is_empty() {
        return Ok(key.to_string());
    }
    for var in ["AIHUBMIX_API_KEY", "OPENAI_API_KEY"] {
        if let Ok(v) = std::env::var(var) {
            let v = v.trim();
            if !v.is_empty() {
                return Ok(v.to_string());
            }
        }
    }
    Err(anyhow!("refine API key is empty (set settings.refine_api_key or AIHUBMIX_API_KEY)"))
}

pub fn has_credential(settings: &Settings) -> bool {
    if !settings.refine_api_key.trim().is_empty() {
        return true;
    }
    for var in ["AIHUBMIX_API_KEY", "OPENAI_API_KEY"] {
        if std::env::var(var).map(|v| !v.trim().is_empty()).unwrap_or(false) {
            return true;
        }
    }
    false
}

fn non_empty_or<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    let value = value.trim();
    if value.is_empty() {
        fallback
    } else {
        value
    }
}

/// Some models occasionally wrap output in ```...``` even when told not to.
/// Strip a single outer code fence if it spans the whole output.
fn strip_wrapping_fences(text: &str) -> String {
    let trimmed = text.trim();
    if !trimmed.starts_with("```") {
        return trimmed.to_string();
    }
    let after_open = match trimmed.find('\n') {
        Some(i) => &trimmed[i + 1..],
        None => return trimmed.to_string(),
    };
    let Some(end) = after_open.rfind("```") else {
        return trimmed.to_string();
    };
    after_open[..end].trim_end().to_string()
}
