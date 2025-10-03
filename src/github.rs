// src/github.rs (Solusi Definitif dengan Login Shell)

use std::process::Command;
use std::fmt;
use std::thread;
use std::time::{Duration, Instant};

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

fn stop_codespace(token: &str, name: &str) -> Result<(), GHError> {
    println!("      Stopping '{}'...", name);
    match run_gh_command(token, &["codespace", "stop", "-c", name]) {
        Ok(_) => { 
            println!("      Stopped"); 
            thread::sleep(Duration::from_secs(5)); 
            Ok(()) 
        }
        Err(e) => { 
            eprintln!("      Warning while stopping: {}", e); 
            thread::sleep(Duration::from_secs(3)); 
            Ok(()) 
        }
    }
}

fn delete_codespace(token: &str, name: &str) -> Result<(), GHError> {
    println!("      Deleting '{}'...", name);
    for attempt in 1..=3 {
        match run_gh_command(token, &["codespace", "delete", "-c", name, "--force"]) {
            Ok(_) => { println!("      Deleted"); thread::sleep(Duration::from_secs(3)); return Ok(()); }
            Err(_) => {
                if attempt < 3 { eprintln!("      Retry {}/3", attempt); thread::sleep(Duration::from_secs(5)); } 
                else { eprintln!("      Failed after 3 attempts, continue anyway"); return Ok(()); }
            }
        }
    }
    Ok(())
}

fn wait_for_deletion(token: &str, repo: &str, timeout_secs: u64) -> Result<(), GHError> {
    println!("      Waiting for old codespaces to be fully deleted...");
    let start_time = Instant::now();
    loop {
        if start_time.elapsed().as_secs() >= timeout_secs {
            return Err(GHError::CommandError("Timeout: Old codespaces were not deleted in time.".to_string()));
        }
        let list_output = run_gh_command(token, &["codespace", "list", "-r", repo, "-q", "."],)?;
        if list_output.trim().is_empty() {
            println!("      All old codespaces confirmed deleted.");
            return Ok(());
        }
        println!("      Still deleting... checking again in 10s.");
        thread::sleep(Duration::from_secs(10));
    }
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
                
                // ==========================================================
                // INI PERUBAHAN KUNCI
                // ==========================================================
                let script_path = "/workspaces/mawari-nexus-blueprint/auto-start.sh";
                let exec_command = format!("bash -l -c 'bash {}'", script_path);

                match run_gh_command(token, &["codespace", "ssh", "-c", name, "--", &exec_command]) {
                    Ok(start_output) => {
                        println!("      Script execution successful.");
                        println!("      Output snippet: {}", start_output.lines().next().unwrap_or(""));
                        
                        // Cukup verifikasi bahwa perintah berhasil, tidak perlu cek file sinyal
                        // karena keberhasilan eksekusi di login shell sudah cukup.
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

pub fn nuke_and_create(token: &str, repo: &str) -> Result<(String, String), GHError> {
    println!("  Scanning existing codespaces...");
    
    let list_output = run_gh_command(token, &["codespace", "list", "-r", repo, "--json", "name,state", "-q", ".[]"])?;

    if !list_output.is_empty() {
        let codespaces: Vec<&str> = list_output.lines().map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
        if !codespaces.is_empty() {
            println!("  Found {} old codespace(s), cleaning...", codespaces.len());
            for cs_json in codespaces {
                if let (Some(name_start), Some(name_end)) = (cs_json.find("\"name\":\""), cs_json.find("\",\"state\"")) {
                    let name = &cs_json[name_start + 8..name_end];
                    let state = if cs_json.contains("\"state\":\"Available\"") || cs_json.contains("\"state\":\"Running\"") { "Running" } else { "Stopped" };
                    println!("    Codespace: {} ({})", name, state);
                    if state == "Running" { stop_codespace(token, name)?; }
                    delete_codespace(token, name)?;
                    thread::sleep(Duration::from_secs(2));
                }
            }
            println!("  Cleanup commands sent.");
            wait_for_deletion(token, repo, 90)?;
        }
    } else {
        println!("  No old codespaces found");
    }
    
    println!("\n  Creating new codespaces...");
    
    println!("    [1/2] Creating mawari-node (basicLinux32gb)...");
    let mawari_name = run_gh_command(token, &[ "codespace", "create", "-r", repo, "-m", "basicLinux32gb", "--display-name", "mawari-node", "--idle-timeout", "240m", "--retention-period", "24h"])?;
    if mawari_name.is_empty() { return Err(GHError::CommandError("Failed to create mawari-node".to_string())); }
    println!("       Mawari: {}", mawari_name);
    thread::sleep(Duration::from_secs(3));
    
    println!("    [2/2] Creating nexus-node (standardLinux32gb)...");
    let nexus_name = run_gh_command(token, &["codespace", "create", "-r", repo, "-m", "standardLinux32gb", "--display-name", "nexus-node", "--idle-timeout", "240m", "--retention-period", "24h"])?;
    if nexus_name.is_empty() { return Err(GHError::CommandError("Failed to create nexus-node".to_string())); }
    println!("       Nexus: {}", nexus_name);
    
    println!("\n  Starting nodes via direct script execution...");
    wait_and_run_startup_script(token, &mawari_name)?;
    thread::sleep(Duration::from_secs(5));
    wait_and_run_startup_script(token, &nexus_name)?;

    Ok((mawari_name, nexus_name))
}

pub fn ssh_command(token: &str, codespace_name: &str, cmd: &str) -> Result<String, GHError> {
    run_gh_command(token, &["codespace", "ssh", "-c", codespace_name, "--", cmd])
}
