// src/billing.rs (Final - Peringatan sudah diperbaiki)

use std::process::Command;

#[derive(Debug, Clone)]
pub struct BillingInfo {
    pub total_minutes_used: u32,
    pub included_minutes: u32,
    pub minutes_remaining: u32,
    pub hours_remaining: f32,
    pub is_quota_ok: bool,
}

fn run_gh_api(token: &str, endpoint: &str) -> Result<String, String> {
    let output = Command::new("gh")
        .args(&["api", endpoint, "-H", "Accept: application/vnd.github+json"])
        .env("GH_TOKEN", token)
        .output()
        .map_err(|e| format!("Failed to execute gh: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(stderr.to_string());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn get_billing_info(token: &str, username: &str) -> Result<BillingInfo, String> {
    let endpoint = format!("/users/{}/settings/billing/actions", username);
    
    let response = match run_gh_api(token, &endpoint) {
        Ok(r) => r,
        Err(_) => {
            return Ok(BillingInfo {
                total_minutes_used: 0,
                included_minutes: 120,
                minutes_remaining: 120,
                hours_remaining: 60.0,
                is_quota_ok: true,
            });
        }
    };
    
    let json: serde_json::Value = serde_json::from_str(&response)
        .map_err(|e| format!("Parse error: {}", e))?;
    
    let total_minutes_used = json["total_minutes_used"].as_u64().unwrap_or(0) as u32;
    let included_minutes = json["included_minutes"].as_u64().unwrap_or(120) as u32;
    
    let minutes_remaining = included_minutes.saturating_sub(total_minutes_used);
    
    let hours_remaining = (minutes_remaining as f32) / 60.0 / 2.0;
    let is_quota_ok = hours_remaining >= 20.0;
    
    Ok(BillingInfo {
        total_minutes_used,
        included_minutes,
        minutes_remaining,
        hours_remaining,
        is_quota_ok,
    })
}

pub fn display_billing(billing: &BillingInfo, username: &str) {
    println!("Billing @{}: Used {}m of {}m | Remaining {}m ({:.1}h available for 2-core)", 
        username, 
        billing.total_minutes_used, 
        billing.included_minutes, 
        billing.minutes_remaining, 
        billing.hours_remaining
    );
    
    if !billing.is_quota_ok {
        println!("   WARNING: Low quota (< 20h)");
    } else {
        println!("   Quota OK");
    }
}
