// main.rs
mod scy_api;
mod scy_console;
mod scy_prompt;
mod scy_setting;

use scy_api::ScyApi;
use scy_setting::SconnySetting;

fn main() {
    let mut setting = SconnySetting::new();
    setting.load_setting();

    let api = ScyApi::new();

    let system_prompt = "Return JSON like: {\"command\":\"...\",\"notes\":\"...\"}";
    let user_prompt = "지금 이 디렉토리에 있는 a.txt, b.txt, c/ 들을 tar.gz로 압축하는 리눅스 명령어를 만들어줘.";

    match api.generate_json(&setting, user_prompt, system_prompt) {
        Ok(json_text) => {
            println!("=== LLM JSON ===");
            println!("{}", json_text);
        }
        Err(e) => {
            eprintln!("API error: {:?}", e);
            eprintln!("Hint: export OPENAI_API_KEY=... (or SCONNY_OPENAI_API_KEY)");
        }
    }
}
