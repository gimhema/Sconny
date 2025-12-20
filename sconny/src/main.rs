// main.rs
mod scy_console;
mod scy_api;
mod scy_prompt;
mod scy_setting;

use scy_console::{parse_console_request_from_args, run_repl_loop, ConsoleMode};
use scy_setting::SconnySetting;

fn main() {
    let mut setting = SconnySetting::new();
    if let Err(e) = setting.load_setting() {
        eprintln!("Setting load error: {}", e);
        return;
    }

    // args 파싱
    let req = match parse_console_request_from_args() {
        Ok(v) => v,
        Err(help_or_error) => {
            eprintln!("{}", help_or_error);
            return;
        }
    };

    // 1) One-shot
    if let Some(r) = req {
        if r.mode == ConsoleMode::OneShot {
            println!("[Captured Request] {}", r.text);
            println!("[Setting] {:?}", setting);
            return;
        }
    }

    // 2) REPL
    let result = run_repl_loop(|line| {
        println!("[Captured Request] {}", line);
        println!("[Setting] {:?}", setting);
        Ok(())
    });

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }
}
