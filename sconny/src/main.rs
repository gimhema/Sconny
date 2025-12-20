// main.rs
mod scy_console;
mod scy_api;
mod scy_prompt;
mod scy_setting;

use scy_console::{parse_console_request_from_args, run_repl_loop, ConsoleMode};

fn main() {
    // 1) Parse args
    let req = match parse_console_request_from_args() {
        Ok(v) => v,
        Err(help_or_error) => {
            // help text or error
            eprintln!("{}", help_or_error);
            return;
        }
    };

    // 2) MVP: just capture the request text and print it
    if let Some(r) = req {
        if r.mode == ConsoleMode::OneShot {
            println!("[Captured Request] {}", r.text);
            return;
        }
    }

    // 3) REPL mode (no args or --repl)
    let result = run_repl_loop(|line| {
        println!("[Captured Request] {}", line);
        Ok(())
    });

    if let Err(e) = result {
        eprintln!("Error: {}", e);
    }
}
