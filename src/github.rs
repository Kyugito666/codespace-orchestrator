// src/github.rs

use std::process::Command;
use std::fmt;
use std::thread;
use std::time::{Duration};

#[derive(Debug)]
pub enum GHError {
    CommandError(String),
    AuthError(String),
}

impl fmt::Display for GHError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GHError::CommandError(e) => write!(f, "Command failed: {}", e),
            GHError::AuthError(e) => write!(f, "Auth error: {}", e),
        }
    }
}

fn run_gh_command(token: &str, args: &[&str]) -> Result<String, GHError> {
    let output = Command::new("gh")
        .args(args)
        .env("GH_TOKEN", token)
        .output()
        .map_err(|e| GHError::CommandError(format!("Failed to execute gh: {}", e)))?;

    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    
    if !output.status.success() {
        if stderr.contains("Bad credentials") 
            || stderr.contains("authentication required")
            || stderr.contains("HTTP 401") {
            return Err(GHError::AuthError(stderr));
        }
        
        if stderr.contains("no codespaces found") || stdout.trim().is_empty() {
            return Ok("".to_string());
        }
        
        return Err(GHError::CommandError(stderr));
    }
    
    Ok(stdout.trim().to_string())
}

pub fn get_username(token: &str) -> Result<String, GHError> {
    run_gh_command(token, &["api", "user", "--jq", ".login"])
}

pub fn verify_codespace(token: &str, name: &str) -> Result<bool, GHError> {
    let state_check = run_gh_command(token, &["codespace", "view", "-c", name, "--json", "state", "-q", ".state"]);
    match state_check {
        Ok(state) if state == "Available" => Ok(true),
        _ => Ok(false),
    }
}

pub fn wait_and_run_startup_script(token: &str, name: &str) -> Result<(), GHError> {
    println!("   Verifying and starting node '{}'...", name);
    
    for attempt in 1..=10 {
        println!("      Attempt {}/10: Checking SSH readiness...", attempt);
        
        match run_gh_command(token, &["codespace", "ssh", "-c", name, "--", "echo 'ready'"]) {
            Ok(output) if output.contains("ready") => {
                println!("      SSH is ready. Executing auto-start script in a login shell...");
                
                let script_path = "/workspaces/mawari-nexus-blueprint/auto-start.sh";
                let exec_command = format!("bash -l -c 'bash {}'", script_path);

                match run_gh_command(token, &["codespace", "ssh", "-c", name, "--", &exec_command]) {
                    Ok(start_output) => {
                        println!("      Script execution successful.");
                        println!("      Output snippet: {}", start_output.lines().next().unwrap_or(""));
                        return Ok(());
                    },
                    Err(e) => {
                        eprintln!("      Error executing auto-start script: {}", e);
                    }
                }
            },
            _ => {
                println!("      Codespace is not yet SSH-ready.");
            }
        }

        if attempt < 10 {
            println!("      Waiting 30 seconds before next attempt...");
            thread::sleep(Duration::from_secs(30));
        }
    }

    Err(GHError::CommandError(format!("Timeout: Failed to reliably start node in '{}' after multiple attempts.", name)))
}

// ==========================================================
// FUNGSI INI MENGGANTIKAN 'nuke_and_create'
// ==========================================================
pub fn find_or_create_codespaces(token: &str, repo: &str) -> Result<(String, String), GHError> {
    println!("  Checking for existing codespaces...");
    
    let mut mawari_name = String::new();
    let mut nexus_name = String::new();

    // 1. Coba cari codespace yang sudah ada berdasarkan display name
    let list_output = run_gh_command(token, &["codespace", "list", "--json", "name,displayName", "-q", ".[]"])?;
    if !list_output.is_empty() {
        for line in list_output.lines() {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
                let display_name = json["displayName"].as_str().unwrap_or("");
                let name = json["name"].as_str().unwrap_or("").to_string();

                if display_name == "mawari-node" {
                    mawari_name = name;
                } else if display_name == "nexus-node" {
                    nexus_name = name;
                }
            }
        }
    }

    // 2. Jika mawari-node tidak ditemukan, buat baru.
    if mawari_name.is_empty() {
        println!("  'mawari-node' not found. Creating new one...");
        let new_name = run_gh_command(token, &[ "codespace", "create", "-r", repo, "-m", "basicLinux32gb", "--display-name", "mawari-node", "--idle-timeout", "240m"])?;
        if new_name.is_empty() { return Err(GHError::CommandError("Failed to create mawari-node".to_string())); }
        mawari_name = new_name;
        println!("     Created: {}", mawari_name);
    } else {
        println!("  Found existing 'mawari-node': {}", mawari_name);
    }
    
    // 3. Jika nexus-node tidak ditemukan, buat baru.
    if nexus_name.is_empty() {
        println!("  'nexus-node' not found. Creating new one...");
        let new_name = run_gh_command(token, &["codespace", "create", "-r", repo, "-m", "standardLinux32gb", "--display-name", "nexus-node", "--idle-timeout", "240m"])?;
        if new_name.is_empty() { return Err(GHError::CommandError("Failed to create nexus-node".to_string())); }
        nexus_name = new_name;
        println!("     Created: {}", nexus_name);
    } else {
        println!("  Found existing 'nexus-node': {}", nexus_name);
    }

    println!("\n  Starting nodes via direct script execution...");
    wait_and_run_startup_script(token, &mawari_name)?;
    thread::sleep(Duration::from_secs(5));
    wait_and_run_startup_script(token, &nexus_name)?;

    Ok((mawari_name, nexus_name))
}

pub fn ssh_command(token: &str, codespace_name: &str, cmd: &str) -> Result<String, GHError> {
    run_gh_command(token, &["codespace", "ssh", "-c", codespace_name, "--", cmd])
}
