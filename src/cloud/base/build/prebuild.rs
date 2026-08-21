use crate::{core::{checks::{Check, CheckIf}, payload::{Payload, Take}, runtime::Nya}, cloud::{types::NodeCommandResult, utils::{get_base_nodes, prepare_base_context, run_on_node}}};
use crate::cloud::{types, utils};
use openssh::Session;
use types::BaseNodeConfig;
use utils::create_ssh_session;

const INSTALL_DOCKER_SCRIPT: &str = include_str!("scripts/install_docker.sh");

pub async fn prebuild_action(nya: Nya, _: Payload) {
  println!("Building the base");
  println!("Running the prebuild");

  prepare_base_context(nya.clone()).await;

  let node_configs: Vec<BaseNodeConfig> = get_base_nodes(nya.clone()).await;

  let mut pre_build_tasks: Vec<(&str, Payload)> = Vec::new();
  for node in node_configs.iter() {
    let session: Session = create_ssh_session(node).await;
    pre_build_tasks.push(("runPreBuild", Payload::new(session)));
  }
  nya.trigger_all(pre_build_tasks).await;
}

pub async fn run_prebuild_script(nya: Nya, payload: Payload) {
  let session: Session = payload.take::<Session>().unwrap();
  let registry_host: String = nya.get("nya.registry_host").await.as_str().unwrap_or("").to_string();
  let daemon_json: String = format!(r#"{{
  "insecure-registries": ["{}"]
}}"#, registry_host);
  let registry_cmd = format!("sudo mkdir -p /etc/docker && echo '{}' | sudo tee /etc/docker/daemon.json", daemon_json);

  if !Check::run(CheckIf::DockerIsInstalled, &session).await {
    let result = run_on_node(&session, INSTALL_DOCKER_SCRIPT).await;
    match result {
      NodeCommandResult::Success => {},
      NodeCommandResult::Failure(err) => { 
        eprintln!("Docker installation failed: {}", err);
        return;
      }
    }

    let registry_result = run_on_node(&session, &registry_cmd).await;
    match registry_result {
      NodeCommandResult::Success => {},
      NodeCommandResult::Failure(err) => {
        eprintln!("Failed to configure Docker registry: {}", err);
        return;
      }
    }
  } else {
    println!("Docker is already installed, skipping installation.");
  }
}