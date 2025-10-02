use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

#[derive(Deserialize)]
pub struct Config {
    pub tokens: Vec<String>,
}

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct State {
    pub current_account_index: usize,
    pub current_mawari_name: String,
    pub current_nexus_name: String,
}

pub fn load_config(path: &str) -> io::Result<Config> {
    if !Path::new(path).exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("File {} tidak ditemukan", path)
        ));
    }
    
    let data = fs::read_to_string(path)?;
    let config: Config = serde_json::from_str(&data)
        .map_err(|e| io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Format JSON salah: {}", e)
        ))?;
    
    if config.tokens.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Array 'tokens' kosong di tokens.json"
        ));
    }
    
    // Validasi format token
    for (i, token) in config.tokens.iter().enumerate() {
        if !token.starts_with("ghp_") && !token.starts_with("github_pat_") {
            eprintln!("⚠️  WARNING: Token #{} mungkin tidak valid (tidak diawali ghp_ atau github_pat_)", i + 1);
        }
    }
    
    Ok(config)
}

pub fn load_state(path: &str) -> io::Result<State> {
    if !Path::new(path).exists() {
        return Ok(State::default());
    }
    
    let data = fs::read_to_string(path)?;
    let state: State = serde_json::from_str(&data)
        .unwrap_or_default();
    Ok(state)
}

pub fn save_state(path: &str, state: &State) -> io::Result<()> {
    let data = serde_json::to_string_pretty(state)?;
    fs::write(path, data)?;
    Ok(())
}
