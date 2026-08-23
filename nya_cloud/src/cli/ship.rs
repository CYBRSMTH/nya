use std::path::PathBuf;
use colored::Colorize;

use crate::core::runtime::Nya;
use crate::cloud::utils_temp::ConfigStatus;
use crate::cloud::utils_temp::{verify_base_config, verify_capsule};

pub async fn run(config: Option<PathBuf>, capsule: Option<PathBuf>) {
  let config_result = verify_base_config(config);
  let nya_base_config_path = match config_result {
    ConfigStatus::Exists(path) => path,
    ConfigStatus::Missing(result) => {
      println!("No config found at {}. Please create a config file to proceed.", result.0.display());
      return;
    }
  };

  let capsule_option = verify_capsule(capsule);
  let nya_capsule_path = match capsule_option {
    ConfigStatus::Exists(path) => path,
    ConfigStatus::Missing(result) => {
      println!("{}{}", "No capsule was found at ".red(), result.0.display().to_string().red());
      return;
    }
  };
  Nya::run("capsule:ship", nya_base_config_path, Some(nya_capsule_path)).await;
}