use serde::Deserialize;
use std::io::{self, Write};
use std::process::Command;

use crate::scy_setting::SconnySetting;

#[derive(Debug, Deserialize)]
pub struct CommandPlan {
    pub cmd: Vec<String>,
    pub explain: Option<String>,
    pub needs_confirmation: Option<bool>,
    pub risk: Option<String>, // "low" | "medium" | "high"
    pub assumptions: Option<Vec<String>>,
    pub notes: Option<Vec<String>>,
}

pub fn handle_plan_json(setting: &SconnySetting, json_text: &str) -> Result<(), String> {
    let plan: CommandPlan = serde_json::from_str(json_text)
        .map_err(|e| format!("Failed to parse LLM JSON: {}", e))?;

    print_plan(&plan);

    if plan.cmd.is_empty() {
        return Err("LLM returned empty cmd list. Aborting.".to_string());
    }

    // dry_run이면 절대 실행 안 함
    if setting.policy.dry_run {
        println!("\n[dry_run=true] Not executing commands.");
        return Ok(());
    }

    // confirmation 정책
    let must_confirm = setting.policy.require_confirmation
        || plan.needs_confirmation.unwrap_or(false);

    if must_confirm {
        if !ask_confirmation(plan.risk.as_deref().unwrap_or("low"))? {
            println!("Cancelled.");
            return Ok(());
        }
    }

    // Linux 기준: sh -lc 로 실행
    // timeout은 coreutils 'timeout'이 있으면 적용
    for (i, c) in plan.cmd.iter().enumerate() {
        println!("\n--- Running ({}/{}) ---\n{}", i + 1, plan.cmd.len(), c);
        run_shell_command_with_timeout(c, setting.policy.timeout_sec)?;
    }

    Ok(())
}

fn print_plan(plan: &CommandPlan) {
    println!("\n=== PLAN ===");
    if let Some(explain) = &plan.explain {
        println!("Explain: {}", explain);
    }
    println!("Risk: {}", plan.risk.as_deref().unwrap_or("unknown"));
    println!("Needs confirmation (plan): {}", plan.needs_confirmation.unwrap_or(false));
    println!("\nCommands:");
    for (i, c) in plan.cmd.iter().enumerate() {
        println!("  {}. {}", i + 1, c);
    }
    if let Some(a) = &plan.assumptions {
        if !a.is_empty() {
            println!("\nAssumptions:");
            for x in a { println!("  - {}", x); }
        }
    }
    if let Some(n) = &plan.notes {
        if !n.is_empty() {
            println!("\nNotes:");
            for x in n { println!("  - {}", x); }
        }
    }
}

fn ask_confirmation(risk: &str) -> Result<bool, String> {
    let risk_lc = risk.trim().to_lowercase();
    if risk_lc == "high" {
        print!("\nRisk is HIGH. Type YES to execute: ");
        io::stdout().flush().map_err(|e| e.to_string())?;
        let mut s = String::new();
        io::stdin().read_line(&mut s).map_err(|e| e.to_string())?;
        return Ok(s.trim() == "YES");
    }

    print!("\nExecute these commands? [y/N]: ");
    io::stdout().flush().map_err(|e| e.to_string())?;
    let mut s = String::new();
    io::stdin().read_line(&mut s).map_err(|e| e.to_string())?;
    let v = s.trim().to_lowercase();
    Ok(v == "y" || v == "yes")
}

fn run_shell_command_with_timeout(cmd: &str, timeout_sec: u64) -> Result<(), String> {
    // timeout 커맨드가 있으면: timeout 15s sh -lc "<cmd>"
    // 없으면: sh -lc "<cmd>"
    let use_timeout = Command::new("sh")
        .arg("-lc")
        .arg("command -v timeout >/dev/null 2>&1")
        .status()
        .map(|s| s.success())
        .unwrap_or(false);

    let mut c = if use_timeout {
        let t = format!("{}s", timeout_sec);
        let mut x = Command::new("timeout");
        x.arg(t).arg("sh").arg("-lc").arg(cmd);
        x
    } else {
        let mut x = Command::new("sh");
        x.arg("-lc").arg(cmd);
        x
    };

    let out = c.output().map_err(|e| e.to_string())?;

    if !out.status.success() {
        return Err(format!(
            "Command failed (code={:?}).\n--- stdout ---\n{}\n--- stderr ---\n{}",
            out.status.code(),
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr),
        ));
    }

    if !out.stdout.is_empty() {
        print!("{}", String::from_utf8_lossy(&out.stdout));
    }
    if !out.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&out.stderr));
    }
    Ok(())
}
