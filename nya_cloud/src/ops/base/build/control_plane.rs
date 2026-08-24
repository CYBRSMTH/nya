use nya_core::{payload::Payload, runtime::Nya};
use crate::utils::{checks::{Check, CheckIf}, types::NodeCommandResult, utils::{get_control_plane_config, get_from_node, run_on_node}};
use crate::utils::{types, utils};
use openssh::Session;
use tera::Context;
use types::BaseNodeConfig;
use utils::create_ssh_session;
use crate::utils::utils::get_node_configs;

const INSTALL_K3S_SCRIPT: &str = include_str!("scripts/install_k3s.sh");
const K3S_REGISTRIES_TEMPLATE: &str = include_str!("templates/registries.yaml");
const INSTALL_HELM_SCRIPT: &str = include_str!("scripts/install_helm.sh");

#[derive(serde::Serialize)]
struct K3sScriptContext {
  control_plane_ip: String,
  k3s_token: String,
  registry_host: String,
}

pub async fn build_control_plane_action(nya: Nya, _: Payload) {
  println!("Building the control plane");

  let control_plane_config: BaseNodeConfig = get_control_plane_config(nya.clone()).await;
  let k3s_token: String = nya.get("k3s_token").await.as_str().unwrap_or("").to_string();
  let registry_host: String = nya.get("nya.registry_host").await.as_str().unwrap_or("").to_string();
  let control_plane_context = K3sScriptContext {
    control_plane_ip: control_plane_config.host.clone(),
    k3s_token,
    registry_host,
  };

  let context_value = serde_json::to_value(&control_plane_context).unwrap();
  let tera_context = Context::from_serialize(&context_value).unwrap();
  let rendered_script = tera::Tera::one_off(INSTALL_K3S_SCRIPT, &tera_context, false).unwrap();
  let rendered_registries = tera::Tera::one_off(K3S_REGISTRIES_TEMPLATE, &tera_context, false).unwrap();

  let session: Session = create_ssh_session(&control_plane_config).await;

  let cidr_result = get_from_node(
      &session,
      "ip route | grep -v default | awk '{print $1}' | head -1"
  ).await;
  match cidr_result {
      Ok(cidr) => nya.set("network_cidr", cidr.trim()).await,
      Err(e) => eprintln!("Failed to detect network CIDR: {}", e)
  }

  if !Check::run(CheckIf::K3sIsInstalled, &session).await {
    let k3s_install_result: NodeCommandResult = run_on_node(&session, &rendered_script).await;
    match k3s_install_result {
      NodeCommandResult::Success => println!("K3s installed successfully on control plane."),
      NodeCommandResult::Failure(err) =>  {
        eprintln!("Failed to install K3s on control plane: {}", err);
        return;
      },
    }

    let create_k3s_dir_cmd = "sudo mkdir -p /etc/rancher/k3s && sudo chmod 755 /etc/rancher/k3s";
    let create_dir_result = run_on_node(&session, create_k3s_dir_cmd).await;
    match create_dir_result {
      NodeCommandResult::Success => println!("Created /etc/rancher/k3s directory successfully."),
      NodeCommandResult::Failure(err) => {
        eprintln!("Failed to create /etc/rancher/k3s directory: {}", err);
        return;
      },
    }

    let registry_cmd = format!("echo '{}' | sudo tee /etc/rancher/k3s/registries.yaml", rendered_registries);
    let registry_result = run_on_node(&session, &registry_cmd).await;
    match registry_result {
      NodeCommandResult::Success => println!("K3s registry configuration applied successfully."),
      NodeCommandResult::Failure(err) => {
        eprintln!("Failed to apply K3s registry configuration: {}", err);
        return;
      },
    }
  } else {
    println!("K3s is already installed on control plane, skipping installation.");
  }

  if !Check::run(CheckIf::HelmIsInstalled, &session).await {
    let helm_install_result: NodeCommandResult = run_on_node(&session, INSTALL_HELM_SCRIPT).await;
    match helm_install_result {
      NodeCommandResult::Success => println!("Helm installed successfully on control plane."),
      NodeCommandResult::Failure(err) => {
        eprintln!("Failed to install Helm on control plane: {}", err);
        return;
      },
    }
  } else {
    println!("Helm is already installed on control plane, skipping installation.");
  }
  
  let node_configs = get_node_configs(nya.clone()).await;
  if node_configs.len() > 0 {
    let get_node_token_cmd = "sudo cat /var/lib/rancher/k3s/server/node-token";
    let token_result = get_from_node(&session, get_node_token_cmd).await;
    match token_result {
      Ok(token) => {
        let _ = nya.set("k3s_node_token", token.trim().to_string()).await;
        println!("Retrieved K3s node token successfully.");
      },
      Err(err) => {
        eprintln!("Failed to retrieve K3s node token: {}", err);
        return;
      },
    }
  }
}