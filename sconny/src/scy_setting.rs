// scy_setting.rs (추가/수정 부분만 발췌)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmService {
    OpenAI,
    Gemini,
    Ollama, // ✅ 추가
}

#[derive(Debug, Clone)]
pub struct SconnySetting {
    pub llm_service: LlmService,
    pub model: Option<String>,

    // ✅ 추가
    pub ollama_base_url: Option<String>,

    pub env: ScyEnvInfo,
    pub policy: ExecPolicy,
    pub config_path: String,
}

impl SconnySetting {
    pub fn new() -> SconnySetting {
        SconnySetting {
            llm_service: LlmService::OpenAI,
            model: None,
            ollama_base_url: None, // ✅ 추가
            env: ScyEnvInfo { /* 동일 */ },
            policy: ExecPolicy { /* 동일 */ },
            config_path: DEFAULT_SCONNY_CONFIG_FILE.to_string(),
        }
    }
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

    // ✅ ollama_base_url
    if let Some(v) = kv.get("ollama_base_url") {
        if !v.trim().is_empty() {
            setting.ollama_base_url = Some(v.trim().to_string());
        }
    }

    // 이하 기존 policy/os/shell 동일...
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

    // ✅ ENV override
    if let Ok(v) = env::var("SCONNY_OLLAMA_BASE_URL") {
        if !v.trim().is_empty() {
            setting.ollama_base_url = Some(v.trim().to_string());
        }
    }

    // 이하 기존 동일...
}

fn parse_llm_service(s: &str) -> Option<LlmService> {
    match s.trim().to_lowercase().as_str() {
        "openai" => Some(LlmService::OpenAI),
        "gemini" => Some(LlmService::Gemini),
        "ollama" => Some(LlmService::Ollama), // ✅ 추가
        _ => None,
    }
}
