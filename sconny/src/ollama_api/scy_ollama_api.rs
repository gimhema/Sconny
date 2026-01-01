use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub enum OllamaError {
    Io(io::Error),
    CommandFailed { code: Option<i32>, stdout: String, stderr: String },
    ParseFailed(&'static str),
}

impl From<io::Error> for OllamaError {
    fn from(e: io::Error) -> Self { OllamaError::Io(e) }
}

pub struct OllamaApi {
    pub base_url: String,   // e.g. http://127.0.0.1:11434
    pub timeout_secs: u64,  // curl --max-time
}

impl OllamaApi {
    pub fn new(base_url: String, timeout_secs: u64) -> Self {
        Self { base_url, timeout_secs }
    }

    /// Ollama /api/chat 호출 (stream=false) -> assistant message.content 반환
    pub fn chat_once_json_only(
        &self,
        model: &str,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String, OllamaError> {
        let url = format!("{}/api/chat", self.base_url.trim_end_matches('/'));

        // Ollama는 messages 배열을 받음.
        // system + user 순으로 넣고, JSON만 출력하라고 system에 강제.
        let body = format!(
            "{{\
\"model\":\"{}\",\
\"stream\":false,\
\"messages\":[\
{{\"role\":\"system\",\"content\":\"{}\"}},\
{{\"role\":\"user\",\"content\":\"{}\"}}\
]\
}}",
            json_escape(model),
            json_escape(system_prompt),
            json_escape(user_prompt),
        );

        let tmp = write_temp_json("sconny_ollama_req", &body)?;
        let raw = call_curl_post_json(&url, &tmp, self.timeout_secs)?;

        // {"message":{"role":"assistant","content":"..."} ...} 에서 content만 추출
        let content = extract_message_content(&raw).ok_or(OllamaError::ParseFailed("failed to extract message.content"))?;
        Ok(content)
    }
}

fn write_temp_json(prefix: &str, content: &str) -> Result<PathBuf, OllamaError> {
    let mut p = env::temp_dir();
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_millis();
    p.push(format!("{}_{}.json", prefix, now));
    fs::write(&p, content.as_bytes())?;
    Ok(p)
}

fn call_curl_post_json(url: &str, body_file: &Path, timeout_secs: u64) -> Result<String, OllamaError> {
    let out = Command::new("curl")
        .arg("-sS")
        .arg("--fail-with-body")
        .arg("--max-time")
        .arg(timeout_secs.to_string())
        .arg(url)
        .arg("-H")
        .arg("Content-Type: application/json")
        .arg("--data-binary")
        .arg(format!("@{}", body_file.display()))
        .output()?;

    if !out.status.success() {
        return Err(OllamaError::CommandFailed {
            code: out.status.code(),
            stdout: String::from_utf8_lossy(&out.stdout).to_string(),
            stderr: String::from_utf8_lossy(&out.stderr).to_string(),
        });
    }

    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

/// 아주 단순 추출기: "message":{"content":"..."}의 content만 뽑음
fn extract_message_content(resp_json: &str) -> Option<String> {
    let needle = "\"message\"";
    let pos = resp_json.find(needle)?;
    let after = &resp_json[pos..];

    let content_key = "\"content\":\"";
    let cpos = after.find(content_key)?;
    let mut i = cpos + content_key.len();

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
                    if i + 4 >= bytes.len() { return None; }
                    let hex = &after[i+1..i+5];
                    if let Ok(v) = u16::from_str_radix(hex, 16) {
                        if let Some(ch) = char::from_u32(v as u32) { out.push(ch); }
                    }
                    i += 4;
                }
                _ => out.push(esc),
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
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}
