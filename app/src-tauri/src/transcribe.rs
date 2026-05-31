use crate::config::{QUERY_URL, SUBMIT_URL};
use crate::settings::Settings;
use anyhow::{anyhow, Result};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde_json::{json, Map, Value};
use std::time::Duration;

pub async fn transcribe(wav: &[u8], settings: Settings) -> Result<String> {
    match settings.transcribe_provider.as_str() {
        "qwen35_omni_plus" => transcribe_qwen_omni(wav, settings, "qwen3.5-omni-plus").await,
        "qwen35_omni_flash" => transcribe_qwen_omni(wav, settings, "qwen3.5-omni-flash").await,
        "qwen3_asr_flash" => transcribe_qwen_asr(wav, settings).await,
        "volc_openspeech" | "" => transcribe_volc_openspeech(wav, settings).await,
        other => Err(anyhow!("unsupported transcribe provider: {}", other)),
    }
}

async fn transcribe_qwen_asr(wav: &[u8], settings: Settings) -> Result<String> {
    let endpoint = dashscope_chat_endpoint(&settings);
    let api_key = dashscope_api_key(&settings)?;

    let mut asr_options = Map::new();
    asr_options.insert("enable_itn".into(), Value::Bool(true));
    let lang = settings.qwen_asr_language.trim();
    if !lang.is_empty() {
        asr_options.insert("language".into(), Value::String(lang.to_string()));
    }

    let model = non_empty_or(&settings.qwen_asr_model, crate::config::QWEN_ASR_MODEL);
    let payload = json!({
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": [
                    {
                        "type": "input_audio",
                        "input_audio": {
                            "data": format!("data:audio/wav;base64,{}", STANDARD.encode(wav)),
                            "format": "wav",
                        },
                    },
                ],
            },
        ],
        "stream": false,
        "asr_options": Value::Object(asr_options),
    });

    let data = post_qwen_json(&endpoint, &api_key, payload).await?;
    let text = data
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .map(content_to_string)
        .unwrap_or_default();
    clean_transcript(&text)
}

async fn transcribe_qwen_omni(wav: &[u8], settings: Settings, model: &str) -> Result<String> {
    let endpoint = dashscope_chat_endpoint(&settings);
    let api_key = dashscope_api_key(&settings)?;

    let prompt = non_empty_or(&settings.omni_prompt, crate::config::OMNI_PROMPT);
    let payload = json!({
        "model": model,
        "messages": [
            {
                "role": "user",
                "content": [
                    {
                        "type": "input_audio",
                        "input_audio": {
                            "data": format!("data:;base64,{}", STANDARD.encode(wav)),
                            "format": "wav",
                        },
                    },
                    {
                        "type": "text",
                        "text": prompt,
                    },
                ],
            },
        ],
        "modalities": ["text"],
        "stream": true,
        "stream_options": {
            "include_usage": false,
        },
    });

    let body = post_qwen_text(&endpoint, &api_key, payload).await?;
    let text = parse_openai_stream_text(&body)?;
    clean_transcript(&text)
}

async fn post_qwen_json(endpoint: &str, api_key: &str, payload: Value) -> Result<Value> {
    let body = post_qwen_text(endpoint, api_key, payload).await?;
    serde_json::from_str(&body).map_err(|e| {
        anyhow!(
            "invalid qwen response: {}; body={}",
            e,
            body.chars().take(600).collect::<String>()
        )
    })
}

async fn post_qwen_text(endpoint: &str, api_key: &str, payload: Value) -> Result<String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(120))
        .build()?;
    let resp = client
        .post(endpoint)
        .bearer_auth(api_key)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await?;
    let status = resp.status();
    let body = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(anyhow!(
            "qwen request failed status={} body={}",
            status,
            body.chars().take(600).collect::<String>()
        ));
    }
    Ok(body)
}

fn parse_openai_stream_text(body: &str) -> Result<String> {
    let mut out = String::new();
    let mut saw_event = false;
    for line in body.lines() {
        let line = line.trim();
        let Some(data) = line.strip_prefix("data:") else {
            continue;
        };
        let data = data.trim();
        if data.is_empty() || data == "[DONE]" {
            continue;
        }
        saw_event = true;
        let chunk: Value = serde_json::from_str(data)
            .map_err(|e| anyhow!("invalid qwen stream chunk: {}; chunk={}", e, data))?;
        if let Some(error) = chunk.get("error") {
            return Err(anyhow!("qwen stream error: {}", error));
        }
        if let Some(delta) = chunk
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("delta"))
        {
            out.push_str(&content_to_string(delta.get("content").unwrap_or(&Value::Null)));
        }
        if let Some(message) = chunk
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
        {
            out.push_str(&content_to_string(
                message.get("content").unwrap_or(&Value::Null),
            ));
        }
    }
    if saw_event {
        Ok(out)
    } else {
        let data: Value = serde_json::from_str(body)
            .map_err(|e| anyhow!("invalid qwen response: {}; body={}", e, body))?;
        Ok(data
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .map(content_to_string)
            .unwrap_or_default())
    }
}

fn content_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Array(parts) => parts.iter().map(content_to_string).collect(),
        Value::Object(obj) => obj
            .get("text")
            .or_else(|| obj.get("content"))
            .map(content_to_string)
            .unwrap_or_default(),
        _ => String::new(),
    }
}

fn clean_transcript(text: &str) -> Result<String> {
    let mut text = text.trim().to_string();
    for prefix in [
        "转写文本：",
        "转写文本:",
        "转写：",
        "转写:",
        "Transcript:",
        "Transcription:",
    ] {
        if let Some(rest) = text.strip_prefix(prefix) {
            text = rest.trim().to_string();
        }
    }
    if text.is_empty() {
        Err(anyhow!("empty transcription result"))
    } else {
        Ok(text)
    }
}

fn dashscope_chat_endpoint(settings: &Settings) -> String {
    let base = non_empty_or(
        &settings.dashscope_base_url,
        crate::config::DASHSCOPE_BASE_URL,
    );
    format!("{}/chat/completions", base.trim_end_matches('/'))
}

fn dashscope_api_key(settings: &Settings) -> Result<String> {
    let key = settings.dashscope_api_key.trim();
    if !key.is_empty() {
        return Ok(key.to_string());
    }
    let key = std::env::var("DASHSCOPE_API_KEY").unwrap_or_default();
    let key = key.trim();
    if key.is_empty() {
        Err(anyhow!("DashScope API key is required"))
    } else {
        Ok(key.to_string())
    }
}

fn non_empty_or<'a>(value: &'a str, fallback: &'a str) -> &'a str {
    let value = value.trim();
    if value.is_empty() {
        fallback
    } else {
        value
    }
}

async fn transcribe_volc_openspeech(wav: &[u8], settings: Settings) -> Result<String> {
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
