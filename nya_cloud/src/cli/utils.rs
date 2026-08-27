use std::env;
use std::path::PathBuf;
use crate::cli::defaults;

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