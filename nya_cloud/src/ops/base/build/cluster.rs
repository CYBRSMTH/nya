use std::sync::Arc;
use colored::*;

use openssh::Session;
use tera::Context;
use serde_json::Value;
use include_dir::{include_dir, Dir};
use base64::{engine::general_purpose, Engine as _};

use nya_core::{payload::{Payload, Take}, runtime::Nya};
use crate::ops::types::{BaseNodeConfig, ClusterBind9Context, NodeCommandResult};
use crate::ops::utils::{create_ssh_session, get_control_plane_config, get_node_configs, run_on_node};
use crate::ops::utils::get_from_node;
use crate::ops::checks::{Check, CheckIf};

const K3S_REGISTRIES_TEMPLATE: &str = include_str!("templates/registries.yaml");
const NAMED_CONF_LOCAL_TEMPLATE: &str = include_str!("templates/named.conf.local");
const NAMED_CONF_OPTIONS_TEMPLATE: &str = include_str!("templates/named.conf.options");
const BIND9_DB_TEMPLATE: &str = include_str!("templates/bind9.db");
const HELM_DIR: Dir = include_dir!("nya_cloud/src/ops/base/build/helm");
const HELM_TEMPLATES_DIR: Dir = include_dir!("nya_cloud/src/ops/base/build/helm/templates");


#[derive(serde::Serialize, Clone, Debug)]
struct K3sAgentScriptContext {
  control_plane_ip: String,
  registry_host: String,
}

#[derive(serde::Serialize, Clone, Debug)]
struct TLSScriptContext {
  domain: String,
}

pub async fn complete_cluster(nya: Nya, _: Payload) {
  let control_plane_config: BaseNodeConfig = get_control_plane_config(nya.clone()).await;
  let nodes_configs: Vec<BaseNodeConfig> = get_node_configs(nya.clone()).await;
  let mut node_triggers: Vec<(&str, Payload)> = Vec::new();
  for node in nodes_configs.iter() {
    let session: Session = create_ssh_session(node).await;
    node_triggers.push(("registerNode", Payload::new((session, node.clone()))));
  }

  let control_plane_session = create_ssh_session(&control_plane_config).await;
  let session_arc = Arc::new(control_plane_session);

  let _ = nya.trigger_all(node_triggers).await;
  let _ = nya.trigger("setupBind9", Payload::new(session_arc.clone())).await;
  let _ = nya.trigger("setupHelm", Payload::new(session_arc.clone())).await;
  let _ = nya.trigger("setupTLS", Payload::new(session_arc.clone())).await;
}

pub async fn on_finish(_nya: Nya, _: Payload) {
  let control_plane_config: BaseNodeConfig = get_control_plane_config(_nya.clone()).await;
  println!("{}", "Build completed successfully!".green());
  println!("To trust the local CA cert, first install mkcert:");
  println!("https://github.com/FiloSottile/mkcert");
  println!("Then run the following to download your cert:");
  println!("ssh {}@{} \"sudo cat /root/.local/share/mkcert/rootCA.pem\" > nya-ca.crt", control_plane_config.user, control_plane_config.host);
  println!("You can now create a Capsule to deploy your apps to Nya by running: {}", "nya capsule new -c ./your_capsule_path".purple());
}

pub async fn register_node(nya: Nya, payload: Payload) {
  let session_obj = payload.take::<(Session, BaseNodeConfig)>().unwrap();
  let control_plane_config: BaseNodeConfig = get_control_plane_config(nya.clone()).await;
  let k3s_token: String = nya.get("k3s_node_token").await.as_str().unwrap().to_string();
  let k3s_install_cmd = format!(
    "curl -sfL https://get.k3s.io | K3S_URL=https://{}:6443 K3S_TOKEN={} sh - > /tmp/k3s-install.log 2>&1",
    control_plane_config.host, k3s_token
  );
  let k3s_wait_cmd = "sudo systemctl is-active --wait k3s-agent";

  let control_plane_context = K3sAgentScriptContext {
    control_plane_ip: control_plane_config.host.clone(),
    registry_host: nya.get("nya.registry_host").await.as_str().unwrap_or("").to_string(),
  };

  let context_value: Value = serde_json::to_value(&control_plane_context).unwrap();
  let tera_context: Context = Context::from_serialize(&context_value).unwrap();
  let rendered_registries = tera::Tera::one_off(K3S_REGISTRIES_TEMPLATE, &tera_context, false).unwrap();

  if !Check::run(CheckIf::K3sAgentIsRunning, &session_obj.0).await {
    let create_k3s_dir_cmd: &str = "sudo mkdir -p /etc/rancher/k3s && sudo chmod 755 /etc/rancher/k3s";
    let create_dir_result: NodeCommandResult = run_on_node(&session_obj.0, create_k3s_dir_cmd).await;
    match create_dir_result {
      NodeCommandResult::Success => println!("Created /etc/rancher/k3s directory successfully."),
      NodeCommandResult::Failure(err) => {
        eprintln!("Failed to create /etc/rancher/k3s directory: {}", err);
        return;
      },
    }

    let encoded_registries = general_purpose::STANDARD.encode(&rendered_registries);
    let registry_cmd = format!(
      "echo '{}' | base64 -d | sudo tee /etc/rancher/k3s/registries.yaml",
      encoded_registries
    );
    let registry_result: NodeCommandResult = run_on_node(&session_obj.0, &registry_cmd).await;
    match registry_result {
      NodeCommandResult::Success => println!("K3s registry configuration applied successfully."),
      NodeCommandResult::Failure(err) => {
        eprintln!("Failed to apply K3s registry configuration: {}", err);
        return;
      },
    }

    println!("Starting K3s agent install on node {} and registering with control plane...", session_obj.1.host);
    let k3s_agent_install_result = run_on_node(&session_obj.0, &k3s_install_cmd).await;
    match k3s_agent_install_result {
      NodeCommandResult::Success => println!("K3s agent install running on node {}.", session_obj.1.host),
      NodeCommandResult::Failure(err) => {
        eprintln!("Failed to start K3s agent install on node {}: {}", session_obj.1.host, err);
        return;
      },
    }

    println!("Waiting for K3s agent install on node {}.", session_obj.1.host);
    let k3s_install_wait_result = run_on_node(&session_obj.0, &k3s_wait_cmd).await;
    match k3s_install_wait_result {
      NodeCommandResult::Success => println!("K3s agent successfully installed and registered on node {}.", session_obj.1.host),
      NodeCommandResult::Failure(err) => {
        eprintln!("K3s install on node {} did not complete successfully: {}", session_obj.1.host, err);
        return;
      },
    }
  } else {
    println!("K3s agent is already running on node {}, skipping installation and registration.", session_obj.1.host);
  }
}

pub async fn setup_bind9(nya: Nya, payload: Payload) {
  let session: Arc<Session> = payload.take::<Arc<Session>>().unwrap();

  let control_plane: Value = nya.get("nya.control_plane").await;
  let control_plane_vars: Value = nya.get("nya.control_plane.vars").await;
  let control_plane_ip: &str = control_plane.get("host").unwrap().as_str().unwrap_or("");
  let network_cidr_value= nya.get("network_cidr").await;
  let network_cidr = network_cidr_value.as_str().unwrap_or("");
  let domain_name: &str = control_plane_vars.get("domain_name").unwrap().as_str().unwrap_or("");
  let bind9_context: ClusterBind9Context = ClusterBind9Context {
    control_plane_ip: control_plane_ip.to_string(),
    network_cidr: network_cidr.to_string(),
    domain_name: domain_name.to_string(),
  };

  let context_value: Value = serde_json::to_value(&bind9_context).unwrap();
  let tera_context: Context = Context::from_serialize(&context_value).unwrap();
  let rendered_local: String = tera::Tera::one_off(NAMED_CONF_LOCAL_TEMPLATE, &tera_context, false).unwrap();
  let rendered_options: String = tera::Tera::one_off(NAMED_CONF_OPTIONS_TEMPLATE, &tera_context, false).unwrap();
  let rendered_db: String = tera::Tera::one_off(BIND9_DB_TEMPLATE, &tera_context, false).unwrap();

  let dpkg_config: &str = "sudo dpkg --configure -a";
  let apt_get: &str = "sudo apt-get install -f -y";

  let install_bind9_cmd: &str = "sudo apt update && sudo apt install -y bind9 bind9utils bind9-doc";
  let configure_bind9_cmd: &str = "sudo mkdir -p /etc/bind/zones && sudo chmod 755 /etc/bind/zones";
  let local_cmd: String = format!("echo '{}' | sudo tee /etc/bind/named.conf.local", rendered_local);
  let options_cmd: String = format!("echo '{}' | sudo tee /etc/bind/named.conf.options", rendered_options);
  let db_cmd: String = format!("echo '{}' | sudo tee /etc/bind/zones/db.{}", rendered_db, bind9_context.domain_name);

  let dpkg_result = run_on_node(&session, dpkg_config).await;
  match dpkg_result {
    NodeCommandResult::Success => println!("dpkg configured successfully."),
    NodeCommandResult::Failure(err) => {
      eprintln!("Failed to configure dpkg: {}", err);
      return;
    },
  }

  let apt_get_result = run_on_node(&session, apt_get).await;
  match apt_get_result {
    NodeCommandResult::Success => println!("apt-get dependencies installed successfully."),
    NodeCommandResult::Failure(err) => {
      eprintln!("Failed to install apt-get dependencies: {}", err);
      return;
    },
  }
  
  let bind9_install_result = run_on_node(&session, install_bind9_cmd).await;
  match bind9_install_result {
    NodeCommandResult::Success => println!("Bind9 installed successfully."),
    NodeCommandResult::Failure(err) => {
      eprintln!("Failed to install Bind9: {}", err);
      return;
    },
  }

  let bind9_configure_result = run_on_node(&session, configure_bind9_cmd).await;
  match bind9_configure_result {
    NodeCommandResult::Success => println!("Bind9 configured successfully."),
    NodeCommandResult::Failure(err) => {
      eprintln!("Failed to configure Bind9: {}", err);
      return;
    },
  }
  
  let bind9_local_result = run_on_node(&session, &local_cmd).await;
  match bind9_local_result {
    NodeCommandResult::Success => println!("Bind9 local configuration applied successfully."),
    NodeCommandResult::Failure(err) => {
      eprintln!("Failed to apply Bind9 local configuration: {}", err);
      return;
    },
  }
  
  let bind9_options_result = run_on_node(&session, &options_cmd).await;
  match bind9_options_result {
    NodeCommandResult::Success => println!("Bind9 options configuration applied successfully."),
    NodeCommandResult::Failure(err) => {
      eprintln!("Failed to apply Bind9 options configuration: {}", err);
      return;
    },
  }

  let bind9_db_result = run_on_node(&session, &db_cmd).await;
  match bind9_db_result {
    NodeCommandResult::Success => println!("Bind9 zone file applied successfully."),
    NodeCommandResult::Failure(err) => {
      eprintln!("Failed to apply Bind9 zone file: {}", err);
      return;
    },
  }
}

pub async fn setup_helm(_: Nya, payload: Payload) {
  let session: Arc<Session> = payload.take::<Arc<Session>>().unwrap();
  for file in HELM_DIR.files() {
    let path: String = format!("/opt/nya/charts/{}", file.path().display());
    let content: &str = file.contents_utf8().unwrap();
    let encoded = general_purpose::STANDARD.encode(content);
    let cmd: String = format!("sudo mkdir -p $(dirname {}) && echo '{}' | base64 -d | sudo tee {}", path, encoded, path);
    run_on_node(&session, &cmd).await;
  }

    for file in HELM_TEMPLATES_DIR.files() {
        let path: String = format!("/opt/nya/charts/templates/{}", file.path().display());
        let content: &str = file.contents_utf8().unwrap();
        let encoded = general_purpose::STANDARD.encode(content);
        let cmd: String = format!("sudo mkdir -p $(dirname {}) && echo '{}' | base64 -d | sudo tee {}", path, encoded, path);
        run_on_node(&session, &cmd).await;
    }

  let k3s_yaml_cmd = "sudo chmod 0644 /etc/rancher/k3s/k3s.yaml && sudo chown $USER:$USER /etc/rancher/k3s/k3s.yaml";
  let _ = run_on_node(&session, k3s_yaml_cmd).await;
}

pub async fn setup_tls(nya: Nya, payload: Payload) {
  let session: Arc<Session> = payload.take::<Arc<Session>>().unwrap();
  let domain_name: Value = nya.get("nya.control_plane.vars").await;
  let domain_name_str: &str = domain_name.get("domain_name").unwrap().as_str().unwrap_or("");
  let tls_context = TLSScriptContext {
    domain: domain_name_str.to_string(),
  };
  let context_value: Value = serde_json::to_value(&tls_context).unwrap();
  let tera_context: Context = Context::from_serialize(&context_value).unwrap();
  let setup_mkcert_cmd: String = tera::Tera::one_off(include_str!("scripts/install_mkcert.sh"), &tera_context, false).unwrap();
  let result: NodeCommandResult = run_on_node(&session, &setup_mkcert_cmd).await;
  match result {
    NodeCommandResult::Success => println!("TLS setup completed successfully."),
    NodeCommandResult::Failure(err) => {
      eprintln!("Failed to set up TLS: {}", err);
      return;
    },
  }
}

pub async fn on_build_complete(nya: Nya, _: Payload) {
  let control_plane_base_config = get_control_plane_config(nya.clone()).await;
  let control_plane_session = create_ssh_session(&control_plane_base_config).await;
  let ingress_ip = get_from_node(
    &control_plane_session,
    "kubectl get svc -n ingress-nginx ingress-nginx-controller -o jsonpath='{.status.loadBalancer.ingress[0].ip}'"
  ).await.unwrap();
  nya.set("ingress_ip", ingress_ip.trim()).await;

  let control_plane_vars: Value = nya.get("nya.control_plane.vars").await;
  let network_cidr_value= nya.get("network_cidr").await;
  let network_cidr = network_cidr_value.as_str().unwrap_or("");
  let domain_name: &str = control_plane_vars.get("domain_name").unwrap().as_str().unwrap_or("");
  let bind9_context: ClusterBind9Context = ClusterBind9Context {
    control_plane_ip: ingress_ip,
    network_cidr: network_cidr.to_string(),
    domain_name: domain_name.to_string(),
  };

  let context_value: Value = serde_json::to_value(&bind9_context).unwrap();
  let tera_context: Context = Context::from_serialize(&context_value).unwrap();
  let rendered_db: String = tera::Tera::one_off(BIND9_DB_TEMPLATE, &tera_context, false).unwrap();
  let encoded_db = general_purpose::STANDARD.encode(&rendered_db);
  let db_cmd = format!(
    "echo '{}' | base64 -d | sudo tee /etc/bind/zones/db.{}",
    encoded_db, bind9_context.domain_name
  );
  let bind9_db_result = run_on_node(&control_plane_session, &db_cmd).await;
  match bind9_db_result {
    NodeCommandResult::Success => println!("Updated Bind9 zone file."),
    NodeCommandResult::Failure(err) => {
      eprintln!("Failed to update Bind9 zone file: {}", err);
    },
  }
  run_on_node(&control_plane_session, "sudo systemctl restart bind9").await;
  run_on_node(&control_plane_session, "sudo systemctl restart k3s").await;
  println!("Restarted k3s on node {}", &control_plane_base_config.host);
}