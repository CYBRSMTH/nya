use serde_json::Value;

#[derive(Debug, Clone)]
pub struct BaseNodeConfig {
  pub host: String,
  pub user: String,
  pub ssh_key_path: String,
}

impl BaseNodeConfig {
  pub fn new(value: Value) -> Self {
    Self { 
    host: value.get("host").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
    user: value.get("user").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
    ssh_key_path: value.get("ssh_private_key_file").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(), }
  }
}

pub enum NodeCommandResult {
  Success,
  Failure(String),
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct ClusterBind9Context {
  pub control_plane_ip: String,
  pub network_cidr: String,
  pub domain_name: String,
}