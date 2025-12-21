use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::scy_setting::{LlmService, ScyOs, SconnySetting};

#[derive(Debug)]
pub enum ScyApiError {
    MissingApiKey,
    Io(io::Error),
    CommandFailed { code: Option<i32>, stdout: String, stderr: String },
    ParseFailed(&'static str),
}

impl From<io::Error> for ScyApiError {
    fn from(e: io::Error) -> Self {
        ScyApiError::Io(e)
    }
}

pub struct ScyApi {
    pub base_url: String,     // default: https://api.openai.com
    pub model: String,        // default: gpt-4.1 (원하면 env로 변경)
    pub timeout_secs: u64,    // curl --max-time
    pub store: bool,          // store=false 권장
}

impl ScyApi {
    pub fn new() -> Self {
        let base_url = env::var("SCONNY_OPENAI_BASE_URL").unwrap_or_else(|_| "https://api.openai.com".to_string());
        let model = env::var("SCONNY_OPENAI_MODEL").unwrap_or_else(|_| "gpt-4.1".to_string());
        let timeout_secs = env::var("SCONNY_OPENAI_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(60);

        Self { base_url, model, timeout_secs, store: false }
    }

    pub fn generate_json(&self, setting: &SconnySetting, user_prompt: &str, system_prompt: &str) -> Result<String, ScyApiError> {
        match setting.llm_service {
            LlmService::OpenAI => self.openai_responses_json(setting, user_prompt, system_prompt),
            LlmService::Gemini => {
                Err(ScyApiError::ParseFailed("Gemini not implemented yet"))
            }
        }
    }

    fn openai_responses_json(&self, setting: &SconnySetting, user_prompt: &str, system_prompt: &str) -> Result<String, ScyApiError> {
        let api_key = get_openai_api_key().ok_or(ScyApiError::MissingApiKey)?;
        let url = format!("{}/v1/responses", self.base_url.trim_end_matches('/'));

        let system = format!(
            "You are a helpful assistant designed to output JSON only.\n\
             Output MUST be a single JSON object. No markdown.\n\n\
             {}",
            system_prompt
        );

        let body = build_responses_body_json(&self.model, &system, user_prompt, self.store);

        let tmp_path = write_temp_json("sconny_openai_req", &body)?;

        let raw = match setting.ScyOs {
            ScyOs::Linux => call_curl_post_json(&url, &api_key, &tmp_path, self.timeout_secs)?,
            ScyOs::Windows => call_powershell_post_json(&url, &api_key, &tmp_path)?,
        };

        // 응답 JSON에서 output_text(content.text)만 추출
        // 공식 레퍼런스의 응답 구조 참고 :contentReference[oaicite:4]{index=4}
        let output_text = extract_first_output_text(&raw).ok_or(ScyApiError::ParseFailed("failed to extract output_text"))?;
        Ok(output_text)
    }
}

fn get_openai_api_key() -> Option<String> {
    // 우선순위: SCONNY_OPENAI_API_KEY -> OPENAI_API_KEY
    env::var("SCONNY_OPENAI_API_KEY")
        .ok()
        .or_else(|| env::var("OPENAI_API_KEY").ok())
        .filter(|s| !s.trim().is_empty())
}

fn build_responses_body_json(model: &str, system_prompt: &str, user_prompt: &str, store: bool) -> String {
    format!(
        "{{\
\"model\":\"{}\",\
\"input\":[\
{{\"role\":\"system\",\"content\":\"{}\"}},\
{{\"role\":\"user\",\"content\":\"{}\"}}\
],\
\"text\":{{\"format\":{{\"type\":\"json_object\"}}}},\
\"store\":{}\
}}",
        json_escape(model),
        json_escape(system_prompt),
        json_escape(user_prompt),
        if store { "true" } else { "false" },
    )
}

fn write_temp_json(prefix: &str, content: &str) -> Result<PathBuf, ScyApiError> {
    let mut p = env::temp_dir();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
    p.push(format!("{}_{}.json", prefix, now));
    fs::write(&p, content.as_bytes())?;
    Ok(p)
}

fn call_curl_post_json(url: &str, api_key: &str, body_file: &Path, timeout_secs: u64) -> Result<String, ScyApiError> {
    let out = Command::new("curl")
        .arg("-sS")
        .arg("--fail-with-body")
        .arg("--max-time")
        .arg(timeout_secs.to_string())
        .arg(url)
        .arg("-H")
        .arg("Content-Type: application/json")
        .arg("-H")
        .arg(format!("Authorization: Bearer {}", api_key))
        .arg("--data-binary")
        .arg(format!("@{}", body_file.display()))
        .output()?;

    if !out.status.success() {
        return Err(ScyApiError::CommandFailed {
            code: out.status.code(),
            stdout: String::from_utf8_lossy(&out.stdout).to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).to_string(),
        });
    }

    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn call_powershell_post_json(url: &str, api_key: &str, body_file: &Path, _timeout_secs: u64) -> Result<String, ScyApiError> {
    let script = format!(
        "$body = Get-Content -Raw '{}'; \
         $headers = @{{ Authorization = 'Bearer {}' }}; \
         $resp = Invoke-RestMethod -Method Post -Uri '{}' -Headers $headers -ContentType 'application/json' -Body $body; \
         $resp | ConvertTo-Json -Depth 30",
        body_file.display(),
        api_key.replace("'", "''"),
        url.replace("'", "''")
    );

    let out = Command::new("powershell")
        .arg("-NoProfile")
        .arg("-Command")
        .arg(script)
        .output()?;

    if !out.status.success() {
        return Err(ScyApiError::CommandFailed {
            code: out.status.code(),
            stdout: String::from_utf8_lossy(&out.stdout).to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).to_string(),
        });
    }

    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn extract_first_output_text(resp_json: &str) -> Option<String> {
    let needle = "\"type\":\"output_text\"";
    let pos = resp_json.find(needle)?;
    let after = &resp_json[pos + needle.len()..];

    let text_key = "\"text\":\"";
    let tpos = after.find(text_key)?;
    let mut i = tpos + text_key.len();

    // JSON string parse (\" \\n \\uXXXX 등)
    let mut out = String::new();
    let bytes = after.as_bytes();
    while i < bytes.len() {
        let c = bytes[i] as char;
        if c == '"' {
            return Some(out);
        }
        if c == '\\' {
            i += 1;
            if i >= bytes.len() { return None; }
            let esc = bytes[i] as char;
            match esc {
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                '/' => out.push('/'),
                'b' => out.push('\u{0008}'),
                'f' => out.push('\u{000C}'),
                'n' => out.push('\n'),
                'r' => out.push('\r'),
                't' => out.push('\t'),
                'u' => {
                    // \uXXXX
                    if i + 4 >= bytes.len() { return None; }
                    let hex = &after[i+1..i+5];
                    if let Ok(v) = u16::from_str_radix(hex, 16) {
                        if let Some(ch) = char::from_u32(v as u32) {
                            out.push(ch);
                        }
                    }
                    i += 4;
                }
                _ => { out.push(esc); }
            }
        } else {
            out.push(c);
        }
        i += 1;
    }
    None
}

fn json_escape(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out
}