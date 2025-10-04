// src/main.rs

mod config;
mod github;
mod billing;

use std::thread;
use std::time::{Duration, Instant};
use std::env;

const STATE_FILE: &str = "state.json";
const KEEP_ALIVE_INTERVAL: Duration = Duration::from_secs(3 * 3600 + 30 * 60); // 3.5 jam

// ... (fungsi show_status, verify_current, dan restart_nodes tidak berubah) ...
fn show_status() {
    println!("STATUS ORCHESTRATOR");
    println!("==========================================");
    
    match config::load_state(STATE_FILE) {
        Ok(state) => {
            println!("State file found");
            println!("Current Token Index: {}", state.current_account_index);
            if !state.current_mawari_name.is_empty() {
                println!("Mawari Node: {}", state.current_mawari_name);
            }
            if !state.current_nexus_name.is_empty() {
                println!("Nexus Node: {}", state.current_nexus_name);
            }
        }
        Err(_) => {
            println!("No state file found");
        }
    }
    
    println!("\nTokens Available:");
    match config::load_config("tokens.json") {
        Ok(cfg) => {
            println!("   Total: {} tokens", cfg.tokens.len());
        }
        Err(e) => {
            eprintln!("   Error loading tokens: {}", e);
        }
    }
}

fn verify_current() {
    println!("VERIFIKASI NODE AKTIF");
    println!("==========================================");
    
    let state = match config::load_state(STATE_FILE) {
        Ok(s) => s,
        Err(_) => {
            eprintln!("No state file found");
            return;
        }
    };
    
    let config = match config::load_config("tokens.json") {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error loading tokens: {}", e);
            return;
        }
    };
    
    if state.current_account_index >= config.tokens.len() {
        eprintln!("Invalid token index");
        return;
    }
    
    let token = &config.tokens[state.current_account_index];
    
    println!("Token Index: {}", state.current_account_index);
    
    if !state.current_mawari_name.is_empty() {
        println!("\nVerifying Mawari: {}", state.current_mawari_name);
        match github::verify_codespace(token, &state.current_mawari_name) {
            Ok(true) => println!("   RUNNING & READY"),
            Ok(false) => println!("   NOT READY or STOPPED"),
            Err(e) => eprintln!("   Error: {}", e),
        }
    }
    
    if !state.current_nexus_name.is_empty() {
        println!("\nVerifying Nexus: {}", state.current_nexus_name);
        match github::verify_codespace(token, &state.current_nexus_name) {
            Ok(true) => println!("   RUNNING & READY"),
            Ok(false) => println!("   NOT READY or STOPPED"),
            Err(e) => eprintln!("   Error: {}", e),
        }
    }
}

fn restart_nodes(token: &str, mawari_name: &str, nexus_name: &str) {
    let script_path = "/workspaces/mawari-nexus-blueprint/auto-start.sh";
    let cmd = format!("bash -l -c 'bash {}'", script_path);

    println!("  Restarting Mawari: {}", mawari_name);
    match github::ssh_command(token, mawari_name, &cmd) {
        Ok(output) => println!("    Restart sent. Output: {}", output.lines().next().unwrap_or("")),
        Err(e) => eprintln!("    Warning: {}", e),
    }
    
    thread::sleep(Duration::from_secs(2));
    
    println!("  Restarting Nexus: {}", nexus_name);
    match github::ssh_command(token, nexus_name, &cmd) {
        Ok(output) => println!("    Restart sent. Output: {}", output.lines().next().unwrap_or("")),
        Err(e) => eprintln!("    Warning: {}", e),
    }
}


fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() > 1 && args[1] == "status" {
        show_status();
        return;
    }
    
    if args.len() > 1 && args[1] == "verify" {
        verify_current();
        return;
    }
    
    if args.len() < 2 {
        eprintln!("Error: Nama repo belum dikasih!");
        eprintln!("Usage: cargo run -- username/nama-repo");
        return;
    }
    
    let repo_name = &args[1];

    println!("==================================================");
    println!("   FULL AUTO ORCHESTRATOR (NUKE & CREATE MODE)");
    println!("==================================================");
    
    println!("\nLoading tokens.json...");
    let config = match config::load_config("tokens.json") {
        Ok(cfg) => cfg,
        Err(e) => {
            eprintln!("FATAL: {}", e);
            return;
        }
    };
    
    println!("Loaded {} tokens", config.tokens.len());
    println!("Target Repo: {}", repo_name);

    let mut state = config::load_state(STATE_FILE).unwrap_or_default();
    let mut i = state.current_account_index;

    if state.current_account_index > 0 {
        println!("Continuing from token index: {}", i);
    }

    println!("\nStarting full auto loop...\n");

    loop {
        let token = &config.tokens[i];
        
        println!("==================================================");
        println!("Token #{} of {}", i + 1, config.tokens.len());
        println!("==================================================");
        
        let username = match github::get_username(token) {
            Ok(u) => {
                println!("Valid token for: @{}", u);
                u
            }
            Err(github::GHError::AuthError(msg)) => {
                eprintln!("Token INVALID: {}", msg.lines().next().unwrap_or(""));
                eprintln!("Skip to next...\n");
                i = (i + 1) % config.tokens.len();
                state.current_account_index = i;
                config::save_state(STATE_FILE, &state).ok();
                thread::sleep(Duration::from_secs(3));
                continue;
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                i = (i + 1) % config.tokens.len();
                state.current_account_index = i;
                config::save_state(STATE_FILE, &state).ok();
                thread::sleep(Duration::from_secs(3));
                continue;
            }
        };

        println!("\nChecking billing quota...");
        let billing = billing::get_billing_info(token, &username).unwrap();
        billing::display_billing(&billing, &username);

        if !billing.is_quota_ok {
            eprintln!("   Kuota tidak cukup. Beralih ke akun berikutnya...\n");
            i = (i + 1) % config.tokens.len();
            state.current_account_index = i;
            config::save_state(STATE_FILE, &state).ok();
            thread::sleep(Duration::from_secs(3));
            continue;
        }

        let (mawari_name, nexus_name) = match github::nuke_and_create(token, repo_name) {
            Ok(names) => names,
            Err(e) => {
                eprintln!("Deployment failed: {}", e);
                eprintln!("Retry in 5 min...\n");
                thread::sleep(Duration::from_secs(5 * 60));
                continue;
            }
        };

        println!("\n==================================================");
        println!("         DEPLOYMENT SUCCESS");
        println!("==================================================");
        println!("Account  : @{}", username);
        println!("Mawari   : {}", mawari_name);
        println!("Nexus    : {}", nexus_name);
        
        state.current_account_index = i;
        state.current_mawari_name = mawari_name.clone();
        state.current_nexus_name = nexus_name.clone();
        config::save_state(STATE_FILE, &state).ok();
        
        println!("State saved");
        
        let run_duration_hours = 20.0;
        let run_duration = Duration::from_secs((run_duration_hours * 3600.0) as u64);
        
        println!("\nRunning for {:.1} hours", run_duration_hours);
        println!("Keep-alive every 3.5 hours");
        
        println!("\nStarting keep-alive loop...\n");
        
        let start_time = Instant::now();
        let mut cycle = 1;
        
        while start_time.elapsed() < run_duration {
            let remaining_duration = run_duration.saturating_sub(start_time.elapsed());
            let sleep_duration = std::cmp::min(KEEP_ALIVE_INTERVAL, remaining_duration);

            if sleep_duration.as_secs() > 60 {
                 println!("\nNext keep-alive in {:.1}h...\n", sleep_duration.as_secs() as f32 / 3600.0);
                 thread::sleep(sleep_duration);
            } else {
                 break;
            }

            if start_time.elapsed() >= run_duration {
                break;
            }
            
            let elapsed_hours = start_time.elapsed().as_secs() / 3600;
            let remaining_hours = (run_duration.as_secs() - start_time.elapsed().as_secs()) / 3600;
            
            println!("--------------------------------------------------");
            println!("Keep-Alive Cycle #{} | Elapsed: ~{}h | Remaining: ~{}h", 
                cycle, elapsed_hours, remaining_hours);
            println!("--------------------------------------------------");
            
            restart_nodes(token, &mawari_name, &nexus_name);
            
            cycle += 1;
        }
        
        println!("\n==================================================");
        println!("Cycle complete! Used {:.1}h", run_duration_hours);
        println!("Switching to next token...");
        println!("==================================================\n");
        
        i = (i + 1) % config.tokens.len();
        state.current_account_index = i;
        config::save_state(STATE_FILE, &state).ok();
        
        if i == 0 {
            println!("Full rotation complete. Back to first token.\n");
        }
    }
}
