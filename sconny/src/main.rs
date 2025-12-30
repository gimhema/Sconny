// main.rs
mod scy_api;
mod scy_console;
mod scy_prompt;
mod scy_setting;
// mod scy_gui; // 차후 추가
mod scy_local_model_loader.rs;

use scy_api::{ScyApi, ScyApiError};
use scy_console::{parse_console_request_from_args, run_repl_loop, ConsoleMode};
use scy_prompt::build_prompt;
use scy_setting::SconnySetting;

fn main() {
    // 1) setting load
    let mut setting = SconnySetting::new();
    if let Err(e) = setting.load_setting() {
        eprintln!("Setting load error: {}", e);
        return;
    }

    // 2) API client
    let api = ScyApi::new();

    // 3) parse console input (oneshot / repl)
    let req = match parse_console_request_from_args() {
        Ok(v) => v,
        Err(help_or_error) => {
            eprintln!("{}", help_or_error);
            return;
        }
    };

    // One-shot
    if let Some(r) = req {
        if r.mode == ConsoleMode::OneShot {
            if let Err(msg) = process_request(&setting, &api, &r.text) {
                eprintln!("{}", msg);
            }
            return;
        }
    }

    // REPL
    let result = run_repl_loop(|line| {
        if let Err(msg) = process_request(&setting, &api, line) {
            eprintln!("{}", msg);
        }
        Ok(()) // 에러가 나도 REPL은 계속
    });

    if let Err(e) = result {
        eprintln!("Console error: {}", e);
    }
}

fn process_request(setting: &SconnySetting, api: &ScyApi, user_text: &str) -> Result<(), String> {
    let user_text = user_text.trim();
    if user_text.is_empty() {
        return Ok(());
    }

    let prompt = build_prompt(setting, user_text).map_err(|e| format!("Prompt build error: {}", e))?;

    if debug_enabled() {
        println!("===== SYSTEM PROMPT =====\n{}\n", prompt.system);
        println!("===== USER PROMPT =====\n{}\n", prompt.user);
    }

    match api.generate_json(setting, &prompt.user, &prompt.system) {
        Ok(json_text) => {
            println!("=== LLM JSON ===");
            println!("{}", json_text);
            Ok(())
        }
        Err(e) => Err(format_api_error(e)),
    }
}

fn debug_enabled() -> bool {
    match std::env::var("SCONNY_DEBUG") {
        Ok(v) => {
            let v = v.trim().to_lowercase();
            v == "1" || v == "true" || v == "yes" || v == "on"
        }
        Err(_) => false,
    }
}

fn format_api_error(e: ScyApiError) -> String {
    match e {
        ScyApiError::MissingApiKey => {
            [
                "API error: Missing API key.",
                "Hint: set OPENAI_API_KEY or SCONNY_OPENAI_API_KEY environment variable.",
            ]
            .join("\n")
        }
        ScyApiError::CommandFailed { code, stdout, stderr } => {
            format!(
                "API error: curl/powershell command failed (code={:?}).\n--- stdout ---\n{}\n--- stderr ---\n{}",
                code, stdout, stderr
            )
        }
        ScyApiError::ParseFailed(msg) => format!("API error: Parse failed: {}", msg),
        ScyApiError::Io(err) => format!("API error: IO error: {}", err),
    }
}
