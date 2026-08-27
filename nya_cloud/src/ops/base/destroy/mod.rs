use nya_core::{payload::{Payload, Take}, runtime::Nya, service::{handle_action, Service, ServiceActions}};
use crate::ops::utils::run_on_node;
use crate::ops::utils;
use openssh::Session;
use serde::Serialize;
use serde_json::Value;
use tera::Context;

const REMOVE_DOCKER_SCRIPT: &str = include_str!("scripts/remove_docker.sh");
const REMOVE_K3S_SERVER_SCRIPT: &str = include_str!("scripts/remove_k3s_server.sh");
const REMOVE_K3S_AGENT_SCRIPT: &str = include_str!("scripts/remove_k3s_agent.sh");
const REMOVE_MKCERT_SCRIPT: &str = include_str!("scripts/remove_mkcert.sh");
const REMOVE_INGRESS_SCRIPT: &str = include_str!("scripts/remove_ingress.sh");
const REMOVE_HELM_SCRIPT: &str = include_str!("scripts/remove_helm.sh");
const REMOVE_BIND9_SCRIPT: &str = include_str!("scripts/remove_bind9.sh");

pub struct NyaBaseDestroy;

use crate::ops::types::BaseNodeConfig;
use utils::create_ssh_session;
use crate::ops::types;

impl Service for NyaBaseDestroy {
  fn name(&self) -> String {"NyaBase".to_string()}
  fn register(&self) -> ServiceActions {
    vec![
      (String::from("onDestroyBase"), handle_action(destroy_action)),
      (String::from("runCleanupNode"), handle_action(run_cleanup_node_script)),
    ]
  }
}

async fn destroy_action(nya: Nya, _: Payload) {
  println!("Destroying the base");

  // Fan out node cleanup in parallel via trigger
  let node_configs: Vec<BaseNodeConfig> = utils::get_node_configs(nya.clone()).await;
  let mut cleanup_tasks: Vec<(&str, Payload)> = Vec::new();
  for node in node_configs.iter() {
    let session = create_ssh_session(node).await;
    cleanup_tasks.push(("runCleanupNode", Payload::new(session)));
  }

  // Control plane teardown in dependency order — blocking, single session
  let control_plane_config: BaseNodeConfig = utils::get_control_plane_config(nya.clone()).await;
  let session = create_ssh_session(&control_plane_config).await;
  let control_plane_vars: Value = nya.get("nya.control_plane.vars").await;
  
  println!("Removing ingress...");
  remove_ingress(&session, &control_plane_vars).await;
  println!("Removing helm...");
  remove_helm(&session).await;
  println!("Removing mkcert...");
  remove_mkcert(&session, &control_plane_vars).await;
  println!("Removing bind9...");
  remove_bind9(&session).await;
  println!("Removing k3s server...");
  remove_k3s_server(&session).await;
  println!("Removing docker...");
  remove_docker(&session).await;
  if let Err(e) = session.close().await {
    eprintln!("destroy_action: failed to close control plane session: {}", e);
  }

  println!("Starting node cleanup...");
  nya.trigger_all(cleanup_tasks).await;
  println!("Node cleanup done");
  // Not sure why the sessions don't close properly after the trigger
  // Putting this here just to let them time out
  // TODO: figure out a better way to ensure sessions are cleaned up after trigger tasks complete
  println!("Finishing up...");
}

async fn run_cleanup_node_script(_: Nya, payload: Payload) {
  let session = payload.take::<Session>().unwrap();

  match run_on_node(&session, REMOVE_K3S_AGENT_SCRIPT).await {
    types::NodeCommandResult::Success => {},
    types::NodeCommandResult::Failure(err) => eprintln!("run_cleanup_node_script: remove k3s-agent script failed: {}", err),
  }

  match run_on_node(&session, REMOVE_DOCKER_SCRIPT).await {
    types::NodeCommandResult::Success => {},
    types::NodeCommandResult::Failure(err) => eprintln!("run_cleanup_node_script: remove docker script failed: {}", err),
  }

  if let Err(e) = session.close().await {
    eprintln!("run_cleanup_node_script: failed to close session: {}", e);
  }
}

#[derive(Serialize)]
struct RemoveIngressContext {
  secret_name: String,
}

async fn remove_ingress(session: &Session, vars: &Value) {
  let secret_name = vars.get("secret_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
  let rendered = tera::Tera::one_off(
    REMOVE_INGRESS_SCRIPT,
    &Context::from_serialize(&serde_json::to_value(RemoveIngressContext { secret_name }).unwrap()).unwrap(),
    false,
  ).unwrap();

  match run_on_node(session, &rendered).await {
    types::NodeCommandResult::Success => {},
    types::NodeCommandResult::Failure(err) => eprintln!("remove_ingress: failed: {}", err),
  }
}

#[derive(Serialize)]
struct RemoveMkcertContext {
  domain: String,
}

async fn remove_mkcert(session: &Session, vars: &Value) {
  let domain = vars.get("domain_name").and_then(|v| v.as_str()).unwrap_or("").to_string();
  let rendered = tera::Tera::one_off(
    REMOVE_MKCERT_SCRIPT,
    &Context::from_serialize(&serde_json::to_value(RemoveMkcertContext { domain }).unwrap()).unwrap(),
    false,
  ).unwrap();

  match run_on_node(session, &rendered).await {
    types::NodeCommandResult::Success => {},
    types::NodeCommandResult::Failure(err) => eprintln!("remove_mkcert: failed: {}", err),
  }
}

async fn remove_helm(session: &Session) {
  match run_on_node(session, REMOVE_HELM_SCRIPT).await {
    types::NodeCommandResult::Success => {},
    types::NodeCommandResult::Failure(err) => eprintln!("remove_helm: failed: {}", err),
  }
}

async fn remove_bind9(session: &Session) {
  match run_on_node(session, REMOVE_BIND9_SCRIPT).await {
    types::NodeCommandResult::Success => {},
    types::NodeCommandResult::Failure(err) => eprintln!("remove_bind9: failed: {}", err),
  }
}

async fn remove_k3s_server(session: &Session) {
  match run_on_node(session, REMOVE_K3S_SERVER_SCRIPT).await {
    types::NodeCommandResult::Success => {},
    types::NodeCommandResult::Failure(err) => eprintln!("remove_k3s_server: failed: {}", err),
  }
}

async fn remove_docker(session: &Session) {
  match run_on_node(session, REMOVE_DOCKER_SCRIPT).await {
    types::NodeCommandResult::Success => {},
    types::NodeCommandResult::Failure(err) => eprintln!("remove_docker: failed: {}", err),
  }
}
