use std::{env, path::PathBuf, process::Stdio};
use tokio::process::Command;

use crate::cloud::defaults;

pub enum ConfigStatus {
    Exists(PathBuf),
    Missing((PathBuf, String)),
}


pub fn verify_base_config(user_input: Option<PathBuf>) -> ConfigStatus {
    let input_path = if let Some(input) = user_input {
        input
    } else {
        defaults::base_config_default_location()
    };
    
    let full_path = if input_path.is_dir() {
        input_path.join(defaults::BASE_CONFIG_DEFAULT_FILE_NAME)
    } else {
        input_path
    };
    
    if !full_path.exists() {
        return ConfigStatus::Missing((full_path, "".to_string()));
    }
    
    ConfigStatus::Exists(full_path)
}

pub fn verify_capsule(user_input: Option<PathBuf>) -> ConfigStatus {
    let fallback_dir;
    let input_path: PathBuf = if let Some(input) = user_input {
        input
    } else {
        fallback_dir = match env::current_dir() {
            Ok(p) => p,
            Err(e) => return ConfigStatus::Missing((PathBuf::new(), e.to_string())),
        };
        fallback_dir
    };

    let full_path = if input_path.is_dir() {
        input_path.join(defaults::CAPSULE_DEFAULT_FILE_DIR_AND_NAME)
    } else {
        input_path
    };

    if !full_path.exists() {
        return ConfigStatus::Missing((full_path, "".to_string()));
    }

    ConfigStatus::Exists(full_path)
}

pub fn generate_sha(location: &str) -> String {
    // Try git sha first
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .current_dir(location)
        .output();
        
    if let Ok(out) = output {
        if out.status.success() {
            return String::from_utf8_lossy(&out.stdout).trim().to_string();
        }
    }
    
    // Fallback to timestamp-based sha
    use std::time::{SystemTime, UNIX_EPOCH};
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    format!("{:x}", ts)[..7].to_string()
}

pub async fn run_ssh(host: &str, user: &str, key: &str, cmd: &str) -> Result<(), String> {
    let mut command = Command::new("ssh");
    command
        .args([
            "-i", key,
            "-o", "StrictHostKeyChecking=no",
            "-o", "UserKnownHostsFile=/dev/null",
            "-o", "IdentitiesOnly=yes",
            &format!("{}@{}", user, host),
            cmd
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = command.output().await.map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).to_string());
    }

    Ok(())
}