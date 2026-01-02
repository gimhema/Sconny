// scy_prompt.rs
use std::env;

use crate::scy_setting::{ScyOs, SconnySetting};

#[derive(Debug, Clone)]
pub struct Prompt {
    pub system: String,
    pub user: String,
}

/// LLM에 전달할 프롬프트(system/user) 생성.
/// - setting.env (OS/배포판/쉘/cwd) 포함
/// - setting.policy (dry_run/confirm/timeout) 포함
/// - 출력은 JSON only 강제
pub fn build_prompt(setting: &SconnySetting, user_request: &str) -> Result<Prompt, String> {
    let cwd = env::current_dir()
        .map_err(|e| format!("failed to get current_dir: {}", e))?
        .display()
        .to_string();

    let os_str = match setting.env.os {
        ScyOs::Linux => "Linux",
        ScyOs::Windows => "Windows",
    };

    let distro = setting
        .env
        .pretty_name
        .clone()
        .or_else(|| setting.env.distro_id.clone())
        .unwrap_or_else(|| "Unknown".to_string());

    let version = setting
        .env
        .version_id
        .clone()
        .unwrap_or_else(|| "Unknown".to_string());

    let shell = setting
        .env
        .shell
        .clone()
        .unwrap_or_else(|| "Unknown".to_string());

    // 정책
    let dry_run = setting.policy.dry_run;
    let require_confirmation = setting.policy.require_confirmation;
    let timeout_sec = setting.policy.timeout_sec;

    // 서비스/모델 정보(프롬프트에 꼭 필요하진 않지만 디버깅에 유용)
    let llm_service = format!("{:?}", setting.llm_service);
    let model = setting
        .model
        .clone()
        .unwrap_or_else(|| "unspecified".to_string());

    // system prompt: "명령 생성기" 역할과 안전 제약을 강하게
    let system = format!(
        concat!(
            "You are Sconny, a safe shell-command generator for a local console assistant.\n",
            "\n",
            "Your job:\n",
            "- Convert the user's natural language request into ONE executable command (or a short list of commands) appropriate for the target environment.\n",
            "- Prefer commands that are widely available on the target OS/distro.\n",
            "- If multiple commands are necessary (e.g., mkdir then tar), keep it minimal.\n",
            "\n",
            "Safety rules (critical):\n",
            "- Do NOT produce destructive or dangerous commands.\n",
            "  Examples of forbidden intent: wiping disks, deleting system files, formatting, fork bombs, privilege escalation, remote code execution.\n",
            "- Avoid anything that can cause irreversible data loss.\n",
            "- If the request is ambiguous or risky, choose the safest interpretation and require confirmation.\n",
            "\n",
            "Output format (MUST follow):\n",
            "- Output JSON ONLY. No markdown, no code fences, no extra text.\n",
            "- Do NOT wrap the JSON in markdown fences like ```json ... ```.\n",
            "- \"cmd\" MUST be an array of FULL shell command strings (one command per string). Do NOT split into argv tokens.\n",
            "- Example cmd: [\"tar -czf archive.tar.gz a.txt b.txt c/\"]\n",
            "- JSON schema:\n",
            "  {{\n",
            "    \"cmd\": [\"<command1>\", \"<command2>\", ...],\n",
            "    \"explain\": \"short explanation\",\n",
            "    \"needs_confirmation\": true|false,\n",
            "    \"risk\": \"low\"|\"medium\"|\"high\",\n",
            "    \"assumptions\": [\"...\"],\n",
            "    \"notes\": [\"...\"]\n",
            "  }}\n",
            "- Always set needs_confirmation=true if policy says confirmation is required.\n",
            "- JSON schema:\n",
            "  {{\n",
            "    \"cmd\": [\"<command1>\", \"<command2>\", ...],\n",
            "    \"explain\": \"short explanation\",\n",
            "    \"needs_confirmation\": true|false,\n",
            "    \"risk\": \"low\"|\"medium\"|\"high\",\n",
            "    \"assumptions\": [\"...\"],\n",
            "    \"notes\": [\"...\"]\n",
            "  }}\n",
            "- Always set needs_confirmation=true if policy says confirmation is required.\n",
            "\n",
            "Environment:\n",
            "- OS: {os}\n",
            "- Distro: {distro}\n",
            "- Version: {version}\n",
            "- Shell: {shell}\n",
            "- CWD: {cwd}\n",
            "\n",
            "Execution policy:\n",
            "- dry_run: {dry_run}\n",
            "- require_confirmation: {require_confirmation}\n",
            "- timeout_sec: {timeout_sec}\n",
            "\n",
            "LLM config (for logging):\n",
            "- llm_service: {llm_service}\n",
            "- model: {model}\n",
        ),
        os = os_str,
        distro = distro,
        version = version,
        shell = shell,
        cwd = cwd,
        dry_run = dry_run,
        require_confirmation = require_confirmation,
        timeout_sec = timeout_sec,
        llm_service = llm_service,
        model = model
    );

    // user prompt: 사용자의 자연어 요청을 그대로
    // 추가로 "명령 후보가 여러개면 어떤 기준으로 선택" 같은 힌트를 더 줄 수도 있음
    let user = format!(
        concat!(
            "User request:\n",
            "{req}\n",
            "\n",
            "Important:\n",
            "- Use the simplest safe command(s).\n",
            "- If target is Linux and task is compressing files/dirs, prefer 'tar' if available.\n",
            "- If the output filename is not specified, choose a sensible default like 'archive.tar.gz'.\n"
        ),
        req = user_request.trim()
    );

    Ok(Prompt { system, user })
}
