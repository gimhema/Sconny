// scy_console.rs
use std::env;
use std::io::{self, Write};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsoleMode {
    OneShot,
    Repl,
}

#[derive(Debug, Clone)]
pub struct ConsoleRequest {
    pub mode: ConsoleMode,
    pub text: String,
}

pub fn parse_console_request_from_args() -> Result<Option<ConsoleRequest>, String> {
    let args: Vec<String> = env::args().collect();

    // args[0] = executable name
    if args.len() <= 1 {
        // no arguments -> REPL mode
        return Ok(None);
    }

    // Simple flags
    if args.len() == 2 && (args[1] == "-h" || args[1] == "--help") {
        return Err(help_text());
    }
    if args.len() == 2 && (args[1] == "--repl" || args[1] == "-i") {
        return Ok(None);
    }

    // One-shot: join all remaining args as the request text
    let text = args[1..].join(" ").trim().to_string();
    if text.is_empty() {
        return Err("Empty request. Try: sconny \"zip a.txt b.txt c/\"".to_string());
    }

    Ok(Some(ConsoleRequest {
        mode: ConsoleMode::OneShot,
        text,
    }))
}

pub fn run_repl_loop<F>(mut on_request: F) -> Result<(), String>
where
    F: FnMut(&str) -> Result<(), String>,
{
    let stdin = io::stdin();

    loop {
        print!("sconny> ");
        io::stdout()
            .flush()
            .map_err(|e| format!("stdout flush error: {}", e))?;

        let mut line = String::new();
        let n = stdin
            .read_line(&mut line)
            .map_err(|e| format!("stdin read error: {}", e))?;

        if n == 0 {
            // EOF (Ctrl+D)
            break;
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line == ":q" || line == ":quit" || line == "exit" {
            break;
        }
        if line == ":help" {
            println!("{}", repl_help_text());
            continue;
        }

        on_request(line)?;
    }

    Ok(())
}

fn help_text() -> String {
    [
        "Sconny - Smart Console Assistant (input capture MVP)",
        "",
        "USAGE:",
        "  sconny \"<natural language request>\"",
        "  sconny --repl",
        "  sconny                (same as --repl)",
        "",
        "EXAMPLES:",
        "  sconny \"지금 이 디렉토리에 있는 a.txt, b.txt, c/ 들을 압축해줘\"",
        "  sconny --repl",
        "",
        "REPL COMMANDS:",
        "  :help   show help",
        "  :q      quit",
        "",
    ]
    .join("\n")
}

fn repl_help_text() -> String {
    [
        "REPL commands:",
        "  :help   show this help",
        "  :q      quit",
        "  exit    quit",
    ]
    .join("\n")
}
