use crate::config::{QUERY_URL, SUBMIT_URL};
use crate::settings::Settings;
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde_json::{json, Value};
use std::time::Duration;

pub async fn transcribe(wav: &[u8], settings: Settings) -> Result<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()?;
    let request_id = uuid::Uuid::new_v4().to_string();
    let audio_b64 = STANDARD.encode(wav);

    let payload = json!({
        "user": { "uid": settings.app_id },
        "audio": {
            "data": audio_b64,
            "format": "wav",
            "language": settings.language,
        },
        "request": {
            "model_name": "bigmodel",
            "enable_itn": true,
            "enable_punc": true,
            "show_utterances": false,
            "enable_speaker_info": false,
        },
    });

    let submit_resp = client
        .post(SUBMIT_URL)
        .header("Content-Type", "application/json")
        .header("X-Api-App-Key", &settings.app_id)
        .header("X-Api-Access-Key", &settings.access_token)
        .header("X-Api-Resource-Id", &settings.resource_id)
        .header("X-Api-Request-Id", &request_id)
        .header("X-Api-Sequence", "-1")
        .json(&payload)
        .send()
        .await?;

    let status = header_str(&submit_resp, "x-api-status-code");
    if status.as_deref() != Some("20000000") {
        let msg = header_str(&submit_resp, "x-api-message").unwrap_or_default();
        let body = submit_resp.text().await.unwrap_or_default();
        return Err(anyhow!(
            "submit failed code={:?} msg={} body={}",
            status,
            msg,
            body.chars().take(200).collect::<String>()
        ));
    }

    for _ in 0..120u32 {
        tokio::time::sleep(Duration::from_secs(1)).await;
        let resp = client
            .post(QUERY_URL)
            .header("Content-Type", "application/json")
            .header("X-Api-App-Key", &settings.app_id)
            .header("X-Api-Access-Key", &settings.access_token)
            .header("X-Api-Resource-Id", &settings.resource_id)
            .header("X-Api-Request-Id", &request_id)
            .header("X-Api-Sequence", "-1")
            .json(&json!({}))
            .send()
            .await?;

        let h_code = header_str(&resp, "x-api-status-code").unwrap_or_default();
        let data: Value = resp.json().await.unwrap_or(Value::Null);
        let body_code = data
            .get("header")
            .and_then(|h| h.get("code"))
            .and_then(|c| c.as_i64())
            .map(|c| c.to_string());
        let code = body_code.as_deref().unwrap_or(&h_code);

        match code {
            "20000000" => {
                let text = data
                    .get("result")
                    .and_then(|r| r.get("text"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string();
                if !text.is_empty() {
                    return Ok(text);
                }
                if body_code.is_some() {
                    return Err(anyhow!("empty transcription result"));
                }
            }
            "20000001" | "20000002" => continue,
            _ => return Err(anyhow!("query failed code={} body={}", code, data)),
        }
    }
    Err(anyhow!("transcribe timeout (120s)"))
}

fn header_str(resp: &reqwest::Response, key: &str) -> Option<String> {
    resp.headers()
        .get(key)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}
