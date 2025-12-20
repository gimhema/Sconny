// scy_setting.rs
use std::collections::HashMap;
use std::env;
use std::fs;

const DEFAULT_SCONNY_CONFIG_FILE: &str = "sconny_config.toml";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmService {
    OpenAI,
    Gemini,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScyOs {
    Windows,
    Linux,
}

#[derive(Debug, Clone)]
pub struct ScyEnvInfo {
    pub os: ScyOs,
    pub distro_id: Option<String>,      // e.g., "ubuntu", "fedora"
    pub version_id: Option<String>,     // e.g., "22.04"
    pub pretty_name: Option<String>,    // e.g., "Ubuntu 22.04.3 LTS"
    pub shell: Option<String>,          // e.g., "/bin/bash"
}

#[derive(Debug, Clone)]
pub struct ExecPolicy {
    pub dry_run: bool,
    pub require_confirmation: bool,
    pub timeout_sec: u64,
}

#[derive(Debug, Clone)]
pub struct SconnySetting {
    pub llm_service: LlmService,
    pub model: Option<String>,
    pub env: ScyEnvInfo,
    pub policy: ExecPolicy,
    pub config_path: String,
}

impl SconnySetting {
    pub fn new() -> SconnySetting {
        SconnySetting {
            llm_service: LlmService::OpenAI,
            model: None,
            env: ScyEnvInfo {
                os: detect_os(),
                distro_id: None,
                version_id: None,
                pretty_name: None,
                shell: None,
            },
            policy: ExecPolicy {
                dry_run: true,               // 안전하게 기본 dry-run
                require_confirmation: true,   // 기본 확인
                timeout_sec: 15,              // 기본 타임아웃
            },
            config_path: DEFAULT_SCONNY_CONFIG_FILE.to_string(),
        }
    }

    pub fn load_setting(&mut self) -> Result<(), String> {
        if let Ok(p) = env::var("SCONNY_CONFIG") {
            if !p.trim().is_empty() {
                self.config_path = p;
            }
        }

        // 1) 파일에서 로드 (있으면)
        if let Ok(contents) = fs::read_to_string(&self.config_path) {
            let kv = parse_loose_toml_kv(&contents);
            apply_kv(self, &kv);
        }

        // 2) ENV 오버라이드
        apply_env_overrides(self);

        // 3) 환경 자동 감지 보강
        fill_env_defaults(&mut self.env);

        Ok(())
    }
}

// -------------------- internal helpers --------------------

fn detect_os() -> ScyOs {
    if cfg!(windows) {
        ScyOs::Windows
    } else {
        ScyOs::Linux
    }
}

fn parse_loose_toml_kv(contents: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for line in contents.lines() {
        let mut s = line.trim().to_string();
        if s.is_empty() { continue; }

        // 주석 제거
        if let Some(pos) = s.find('#') {
            s.truncate(pos);
            s = s.trim().to_string();
            if s.is_empty() { continue; }
        }

        // 섹션([xxx])은 무시
        if s.starts_with('[') && s.ends_with(']') {
            continue;
        }

        let eq = match s.find('=') {
            Some(p) => p,
            None => continue,
        };

        let key = s[..eq].trim().to_string();
        let mut val = s[eq + 1..].trim().to_string();
        if key.is_empty() || val.is_empty() { continue; }

        val = strip_quotes(&val);
        map.insert(key, val);
    }

    map
}

fn strip_quotes(s: &str) -> String {
    let t = s.trim();
    if (t.starts_with('"') && t.ends_with('"')) || (t.starts_with('\'') && t.ends_with('\'')) {
        if t.len() >= 2 {
            return t[1..t.len() - 1].to_string();
        }
    }
    t.to_string()
}

fn apply_kv(setting: &mut SconnySetting, kv: &HashMap<String, String>) {
    // llm_service
    if let Some(v) = kv.get("llm_service") {
        if let Some(svc) = parse_llm_service(v) {
            setting.llm_service = svc;
        }
    }

    // model
    if let Some(v) = kv.get("model") {
        if !v.trim().is_empty() {
            setting.model = Some(v.trim().to_string());
        }
    }

    // os (강제 지정 가능)
    if let Some(v) = kv.get("os") {
        if let Some(os) = parse_os(v) {
            setting.env.os = os;
        }
    }

    // shell
    if let Some(v) = kv.get("shell") {
        if !v.trim().is_empty() {
            setting.env.shell = Some(v.trim().to_string());
        }
    }

    // policy
    if let Some(v) = kv.get("dry_run") {
        if let Some(b) = parse_bool(v) {
            setting.policy.dry_run = b;
        }
    }
    if let Some(v) = kv.get("require_confirmation") {
        if let Some(b) = parse_bool(v) {
            setting.policy.require_confirmation = b;
        }
    }
    if let Some(v) = kv.get("timeout_sec") {
        if let Ok(n) = v.trim().parse::<u64>() {
            setting.policy.timeout_sec = n;
        }
    }
}

fn apply_env_overrides(setting: &mut SconnySetting) {
    if let Ok(v) = env::var("SCONNY_LLM_SERVICE") {
        if let Some(svc) = parse_llm_service(&v) {
            setting.llm_service = svc;
        }
    }
    if let Ok(v) = env::var("SCONNY_MODEL") {
        if !v.trim().is_empty() {
            setting.model = Some(v.trim().to_string());
        }
    }
    if let Ok(v) = env::var("SCONNY_OS") {
        if let Some(os) = parse_os(&v) {
            setting.env.os = os;
        }
    }
    if let Ok(v) = env::var("SCONNY_SHELL") {
        if !v.trim().is_empty() {
            setting.env.shell = Some(v.trim().to_string());
        }
    }
    if let Ok(v) = env::var("SCONNY_DRY_RUN") {
        if let Some(b) = parse_bool(&v) {
            setting.policy.dry_run = b;
        }
    }
    if let Ok(v) = env::var("SCONNY_CONFIRM") {
        if let Some(b) = parse_bool(&v) {
            setting.policy.require_confirmation = b;
        }
    }
    if let Ok(v) = env::var("SCONNY_TIMEOUT_SEC") {
        if let Ok(n) = v.trim().parse::<u64>() {
            setting.policy.timeout_sec = n;
        }
    }
}

fn parse_bool(s: &str) -> Option<bool> {
    match s.trim().to_lowercase().as_str() {
        "1" | "true" | "yes" | "y" | "on" => Some(true),
        "0" | "false" | "no" | "n" | "off" => Some(false),
        _ => None,
    }
}

fn parse_llm_service(s: &str) -> Option<LlmService> {
    match s.trim().to_lowercase().as_str() {
        "openai" => Some(LlmService::OpenAI),
        "gemini" => Some(LlmService::Gemini),
        _ => None,
    }
}

fn parse_os(s: &str) -> Option<ScyOs> {
    match s.trim().to_lowercase().as_str() {
        "windows" | "win" => Some(ScyOs::Windows),
        "linux" => Some(ScyOs::Linux),
        _ => None,
    }
}

fn fill_env_defaults(envinfo: &mut ScyEnvInfo) {
    // shell
    if envinfo.shell.is_none() {
        if envinfo.os == ScyOs::Windows {
            // Windows: COMSPEC가 보통 "C:\Windows\System32\cmd.exe"
            if let Ok(v) = env::var("COMSPEC") {
                if !v.trim().is_empty() {
                    envinfo.shell = Some(v);
                }
            }
        } else {
            // Linux: SHELL
            if let Ok(v) = env::var("SHELL") {
                if !v.trim().is_empty() {
                    envinfo.shell = Some(v);
                }
            }
        }
    }

    // distro info (Linux only)
    if envinfo.os == ScyOs::Linux
        && (envinfo.distro_id.is_none() || envinfo.version_id.is_none() || envinfo.pretty_name.is_none())
    {
        if let Ok(osr) = fs::read_to_string("/etc/os-release") {
            let oskv = parse_os_release(&osr);
            if envinfo.distro_id.is_none() {
                envinfo.distro_id = oskv.get("ID").cloned();
            }
            if envinfo.version_id.is_none() {
                envinfo.version_id = oskv.get("VERSION_ID").cloned();
            }
            if envinfo.pretty_name.is_none() {
                envinfo.pretty_name = oskv.get("PRETTY_NAME").cloned();
            }
        }
    }
}

fn parse_os_release(contents: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in contents.lines() {
        let s = line.trim();
        if s.is_empty() || s.starts_with('#') { continue; }
        let eq = match s.find('=') { Some(p) => p, None => continue };
        let key = s[..eq].trim().to_string();
        let mut val = s[eq + 1..].trim().to_string();
        val = strip_quotes(&val);
        map.insert(key, val);
    }
    map
}
