use std::path::PathBuf;
use serde_json::{from_str, Value};

pub fn base_config_default_location() -> PathBuf {
    dirs::home_dir()
        .expect("could not determine home directory")
        .join(".nya")
        .join("nya_base_config.json")
}

pub const BASE_CONFIG_DEFAULT_FILE_NAME: &str = "nya_base_config.json";

pub const CAPSULE_DEFAULT_FILE_DIR_AND_NAME: &str = ".nya/nya.json";

pub const NYA_CLOUD_PLANS: &str = include_str!("nya_cloud_plans.json");

pub fn get_cloud_plans() -> Value {
  from_str(NYA_CLOUD_PLANS).unwrap()
}