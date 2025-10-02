mod config;
mod github;

use std::thread;
use std::time::Duration;
use std::env;

const RUN_DURATION: Duration = Duration::from_secs(20 * 3600);
const STATE_FILE: &str = "state.json";

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
            println!("No state file found (belum pernah run)");
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
        eprintln!("Invalid token index in state");
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
        eprintln!("Contoh: cargo run -- username/nama-repo");
        eprintln!("\nCommands:");
        eprintln!("   cargo run -- status   -> Lihat status");
        eprintln!("   cargo run -- verify   -> Verifikasi nodes");
        return;
    }
    let repo_name = &args[1];

    println!("==================================================");
    println!("   ORCHESTRATOR - NUKE & CREATE STRATEGY");
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

    println!("\nStarting loop...\n");

    loop {
        let token = &config.tokens[i];
        
        println!("--------------------------------------------------");
        println!("Token #{} of {}", i + 1, config.tokens.len());
        
        let username = match github::get_username(token) {
            Ok(u) => {
                println!("Valid token for: @{}", u);
                u
            }
            Err(github::GHError::AuthError(msg)) => {
                eprintln!("Token #{} INVALID", i + 1);
                eprintln!("   {}", msg.lines().next().unwrap_or(""));
                eprintln!("Skip to next...\n");
                thread::sleep(Duration::from_secs(3));
                i = (i + 1) % config.tokens.len();
                state.current_account_index = i;
                config::save_state(STATE_FILE, &state).ok();
                continue;
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                eprintln!("Skip...\n");
                thread::sleep(Duration::from_secs(3));
                i = (i + 1) % config.tokens.len();
                state.current_account_index = i;
                config::save_state(STATE_FILE, &state).ok();
                continue;
            }
        };

        println!("\nDeploying for @{}...", username);
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
        println!("\nRunning for 20 hours...");
        println!("Sleeping...\n");
        
        thread::sleep(RUN_DURATION);
        
        println!("\n20 hours completed!");
        println!("Switching to next token...\n");
        
        i = (i + 1) % config.tokens.len();
        state.current_account_index = i;
        config::save_state(STATE_FILE, &state).ok();
        
        if i == 0 {
            println!("Cycle complete. Back to first token.\n");
        }
    }
}
