use std::fs::read_to_string;
use std::path::PathBuf;
use anyhow::Context;
use serde_json::Value;
use nya_core::payload::Payload;
use nya_core::runtime::Nya;
use crate::defaults::get_cloud_plans;
use crate::ops::get_cloud_services;
use anyhow::Result;

pub fn get_json_from_paths(paths: Vec<PathBuf>) -> Result<Vec<Value>> {
  let mut config_jsons = Vec::<Value>::new();

  for path in paths {
    let config_content = read_to_string(&path)
        .context(format!("Failed to read context file '{}'", path.display()))?;
    let config_json: Value = serde_json::from_str(&config_content)
        .context(format!("Failed to parse {}", path.display()))?;
    config_jsons.push(config_json);
  };
  Ok(config_jsons)
}

pub async fn run_nya_command(cmd: &str, relevant_paths: Vec<PathBuf>, payload: Option<String>) -> Result<()> {
  let cloud_services = get_cloud_services();
  let mut context_values = get_json_from_paths(relevant_paths)?;
  context_values.push(get_cloud_plans()?);
  let current_payload = match payload {
    Some(p) => Payload::new(p),
    None => Payload::empty(),
  };
  
  Nya::run(cmd, context_values, cloud_services, current_payload).await?;
  Ok(())
}