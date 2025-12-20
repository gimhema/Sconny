// main.rs
mod scy_console;
mod scy_api;
mod scy_prompt;
mod scy_setting;

use scy_console::{parse_console_request_from_args, run_repl_loop, ConsoleMode};
use scy_prompt::build_prompt;
use scy_setting::SconnySetting;

fn main() {
    let mut setting = SconnySetting::new();
    if let Err(e) = setting.load_setting() {
        eprintln!("Setting load error: {}", e);
        return;
    }

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
            println!("[Captured Request] {}", r.text);

            let prompt = match build_prompt(&setting, &r.text) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Prompt build error: {}", e);
                    return;
                }
            };

            println!("\n===== SYSTEM PROMPT =====\n{}\n", prompt.system);
            println!("===== USER PROMPT =====\n{}\n", prompt.user);
            return;
        }
    }

    // REPL
    let result = run_repl_loop(|line| {
        println!("[Captured Request] {}", line);

        let prompt = build_prompt(&setting, line)?;
        println!("\n===== SYSTEM PROMPT =====\n{}\n", prompt.system);
        println!("===== USER PROMPT =====\n{}\n", prompt.user);

        Ok(())
    });

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }
}
