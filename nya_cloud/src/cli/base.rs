use std::path::PathBuf;
use nya_core::{runtime::Nya, payload::Payload};
use crate::utils::utils::{verify_base_config, ConfigStatus};
use anyhow::{Result, bail};
use serde::{Deserialize, Serialize};
use utils::defaults::default_plans_location;
use crate::utils;

#[derive(Debug, Serialize, Deserialize)]
struct BaseCommandPayload {
  base_config_path: String,
}

pub async fn build(config: Option<PathBuf>) -> Result<()> {
  let input_path = verify_base_config(config);
  let path = match input_path {
    ConfigStatus::Exists(path) => path,
    ConfigStatus::Missing(result) => {
      bail!("No config found at {}. Please create a config file to proceed.", result.0.display());
    },
  };
  let plans_path = default_plans_location();
  let base_command_payload = get_base_command_payload(path.clone())?;
  let payload = Payload::new(base_command_payload);
  Nya::run("base:build", vec![path, plans_path], payload).await?;
  Ok(())
}

pub async fn destroy(config: Option<PathBuf>) -> Result<()> {
  let valid_path = verify_base_config(config);
  let path = match valid_path {
    ConfigStatus::Exists(path) => path,
    ConfigStatus::Missing(result) => {
      bail!("No config found at {}. Please create a config file to proceed.", result.0.display());
    }
  };
  let plans_path = default_plans_location();
  let base_command_payload = get_base_command_payload(path.clone())?;
  let payload = Payload::new(base_command_payload);
  Nya::run("base:destroy", vec![path, plans_path], payload).await?;
  Ok(())
}

fn get_base_command_payload(base_path: PathBuf) -> Result<String> {

  let base_path_string = base_path.to_str().unwrap().to_string();
  let base_command_payload = BaseCommandPayload {
    base_config_path: base_path_string,
  };
  Ok(serde_json::to_string(&base_command_payload)?)
}