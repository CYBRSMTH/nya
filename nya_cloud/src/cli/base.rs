// TODO: The core library no longer cares about the PaaS
// TODO: so the CLI will now have to provide the capsule and base config paths
// TODO: by injecting them into the context at runtime.

use std::path::PathBuf;
use nya_core::runtime::Nya;
use crate::utils::utils_temp::{verify_base_config, ConfigStatus};

pub async fn build(config: Option<PathBuf>) {
  let input_path = verify_base_config(config);
  let path = match input_path {
    ConfigStatus::Exists(path) => path,
    ConfigStatus::Missing(result) => {
      println!("No config found at {}. Please create a config file to proceed.", result.0.display());
      return;
    },
  };
  Nya::run("base:build", path, None).await;
}

pub async fn destroy(config: Option<PathBuf>) {
  let valid_path = verify_base_config(config);
  let path = match valid_path {
    ConfigStatus::Exists(path) => path,
    ConfigStatus::Missing(result) => {
      println!("No config found at {}. Please create a config file to proceed.", result.0.display());
      return;
    }
  };
  Nya::run("base:destroy", path, None).await;
}