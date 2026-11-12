use crate::cli::utils::ConfigStatus;
use crate::cli::utils::{verify_base_config, verify_capsule};
use anyhow::{bail, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::runner::run_nya_command;

#[derive(Debug, Serialize, Deserialize)]
struct ShipCommandPayload {
  base_config_path: String,
  capsule_path: String,
}

pub async fn run(config: Option<PathBuf>, capsule: Option<PathBuf>) -> Result<()> {
  let config_result = verify_base_config(config);
  let nya_base_config_path = match config_result {
    ConfigStatus::Exists(path) => path,
    ConfigStatus::Missing(result) => {
      bail!("No config found at {}. Please create a config file to proceed.", result.0.display());
    }
  };

  let capsule_option = verify_capsule(capsule);
  let nya_capsule_path = match capsule_option {
    ConfigStatus::Exists(path) => path,
    ConfigStatus::Missing(result) => {
      bail!("{}{}", "No capsule was found at ".red(), result.0.display().to_string().red());
    }
  };

  let base_path_string = nya_base_config_path.to_str().unwrap().to_string();
  let capsule_path_string = nya_capsule_path.to_str().unwrap().to_string();

  let ship_command_payload = ShipCommandPayload {
    base_config_path: base_path_string,
    capsule_path: capsule_path_string,
  };
  let ship_command_payload_json = serde_json::to_string(&ship_command_payload)?;
  run_nya_command("capsule:ship", vec![nya_base_config_path, nya_capsule_path], Some(ship_command_payload_json)).await?;
  Ok(())
}