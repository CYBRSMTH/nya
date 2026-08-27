use std::path::PathBuf;

pub fn base_config_default_location() -> PathBuf {
    dirs::home_dir()
        .expect("could not determine home directory")
        .join(".nya")
        .join("nya_base_config.json")
}

pub const BASE_CONFIG_DEFAULT_FILE_NAME: &str = "nya_base_config.json";

pub const CAPSULE_DEFAULT_FILE_DIR_AND_NAME: &str = ".nya/nya.json";

pub const NYA_CLOUD_PLANS_FILE_LOCATION: &str =
  concat!(env!("CARGO_MANIFEST_DIR"), "/nya_cloud_plans.json");

pub fn default_plans_location() -> PathBuf {
  PathBuf::from(NYA_CLOUD_PLANS_FILE_LOCATION)
}
