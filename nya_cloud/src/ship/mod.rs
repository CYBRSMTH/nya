use std::{env, path::PathBuf, process::Stdio};
use futures::future::join_all;
use serde::{Deserialize, Serialize};
use tokio::{io::{AsyncBufReadExt, BufReader}, process::Command};
use crate::{core::{payload::Payload, runtime::Nya, service::{Service, ServiceActions, handle_action}}, cloud::utils::prepare_base_context};

pub struct NyaShip;

impl Service for NyaShip {
  fn name(&self) -> String {"NyaShip".to_string()}
  fn register(&self) -> ServiceActions {
    vec![
      ("onBuildPacks".to_string(), handle_action(build_packs)),
      ("onDeployCapsule".to_string(), handle_action(deploy_capsule)),
    ]
  }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct PackContext {
  pack_name: String,
  pack_location: String,
  pack_image_name: String,
}

async fn build_packs(nya: Nya, _: Payload) {
  let _ = prepare_base_context(nya.clone()).await;
  let registry_host_value = nya.get("nya.registry_host").await;
  let registry_host = registry_host_value.as_str().unwrap_or("");
  let capsule = nya.get("capsule").await;
  let packs = capsule["packs"].as_array().unwrap();
  let full_capsule_path = nya.capsule_path().unwrap();
  let mut full_capsule_path_buf = PathBuf::from(full_capsule_path);
  full_capsule_path_buf.pop();
  full_capsule_path_buf.pop();
  let capsule_path = full_capsule_path_buf.iter().as_path();

  let mut pack_ctx: Vec<PackContext> = vec![];
  let mut build_tasks = vec![];

  for pack in packs.iter() {
    let pack_name = pack["name"].as_str().unwrap().to_string();
    let pack_location = pack["location"].as_str().unwrap();
    let full_path = capsule_path.join(pack_location);
    let path_str = full_path.display().to_string();
    let image_name = format!("{}/{}:latest", registry_host, pack_name);

    let pack_context = PackContext {
      pack_name: pack_name.clone(),
      pack_location: path_str.clone(),
      pack_image_name: image_name.clone(),
    };
    pack_ctx.push(pack_context);

    let nya = nya.clone();
    let build_task = tokio::spawn({
      async move {
        build_cmd(&path_str, &image_name, nya).await
      }
    });
    build_tasks.push(build_task);
  }

  let _ = nya.set("pack_contexts", pack_ctx).await;
  join_all(build_tasks).await;

}

async fn build_cmd(file_path: &str, image_name: &str, nya: Nya) {
  let cmd_args = vec!["build", "-t", image_name, file_path];
  let mut cmd = Command::new("docker");
  cmd.args(cmd_args)
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());

  let mut child = cmd.spawn().unwrap();

  let stdout = child.stdout.take().unwrap();
  let stderr = child.stderr.take().unwrap();

  let mut out_reader = BufReader::new(stdout).lines();
  let mut err_reader = BufReader::new(stderr).lines();

  // Pump stdout
  let out_task = tokio::spawn({
      let nya = nya.clone();
      async move {
          while let Ok(Some(line)) = out_reader.next_line().await {
            let _ = nya.trigger("log", Payload::new(line.clone())).await;
          }
      }
  });

  // Pump stderr
  let err_task = tokio::spawn({
      let nya = nya.clone();
      async move {
          while let Ok(Some(line)) = err_reader.next_line().await {
              let _ = nya.trigger("log", Payload::new(line.clone())).await;
          }
      }
  });

  let status = child.wait().await.unwrap();

  let _ = out_task.await;
  let _ = err_task.await;

  if !status.success() {
      println!("nya:ship failed with {}", status);
  }
}

async fn deploy_capsule(nya: Nya, _: Payload) {
    let pack_contexts_val = nya.get("pack_contexts").await;
    let pack_contexts: Vec<PackContext> = pack_contexts_val
        .as_array()
        .unwrap()
        .iter()
        .map(|v| serde_json::from_value(v.clone()).unwrap())
        .collect();
    
    let mut deploy_tasks = vec![];
    
    for ctx in pack_contexts {
        let nya = nya.clone();
        let deploy_task = tokio::spawn(async move {
            // Push image
            push_image(&ctx.pack_image_name, nya.clone()).await;
            
            // Copy values
            copy_values(&ctx, nya.clone()).await;
            
            // Helm deploy
            helm_deploy(&ctx, nya.clone()).await;
        });
        deploy_tasks.push(deploy_task);
    }
    
    join_all(deploy_tasks).await;
}

async fn push_image(image_name: &str, nya: Nya) {
    let _ = nya.trigger("log", Payload::new(format!("Pushing {}...", image_name))).await;
    
    let mut cmd = Command::new("docker");
    cmd.args(["push", image_name])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    
    run_command(cmd, nya).await;
}

async fn copy_values(ctx: &PackContext, nya: Nya) {
    let control_plane = nya.get("nya.control_plane").await;
    let host = control_plane["host"].as_str().unwrap();
    let user = control_plane["user"].as_str().unwrap();
    let ssh_key = control_plane["ssh_private_key_file"].as_str().unwrap();
    let ssh_key = shellexpand::tilde(ssh_key).to_string();
    
    let values_src = format!("{}/values.yaml", ctx.pack_location);
    let values_dest = format!("/tmp/{}-values.yaml", ctx.pack_name);
    let scp_dest = format!("{}@{}:{}", user, host, values_dest);
    
    let mut cmd = Command::new("scp");
    cmd.args(["-i", &ssh_key, &values_src, &scp_dest])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    
    run_command(cmd, nya).await;
}

async fn helm_deploy(ctx: &PackContext, nya: Nya) {
    let base_vars = nya.get("nya.control_plane.vars").await;
    let control_plane = nya.get("nya.control_plane").await;
    let registry_host_value = nya.get("nya.registry_host").await;
    
    let host = control_plane["host"].as_str().unwrap();
    let user = control_plane["user"].as_str().unwrap();
    let ssh_key = control_plane["ssh_private_key_file"].as_str().unwrap();
    let ssh_key = shellexpand::tilde(ssh_key).to_string();
    
    let registry_host = registry_host_value.as_str().unwrap();
    let domain = base_vars["domain_name"].as_str().unwrap();
    let secret_name = base_vars["secret_name"].as_str().unwrap();

    
    let values_path = format!("/tmp/{}-values.yaml", ctx.pack_name);
    let helm_cmd = format!(
        "helm upgrade --install {} /opt/nya/charts -f {} \
        --set image.name={} \
        --set registry_host={} \
        --set domain={} \
        --set secret_name={} \
        --set podAnnotations.deployedAt='{}' \
        --kubeconfig=/etc/rancher/k3s/k3s.yaml&& \
        kubectl rollout status deployment/{} --timeout=120s --kubeconfig=/etc/rancher/k3s/k3s.yaml",
        ctx.pack_name, values_path,
        ctx.pack_name,
        registry_host, domain, secret_name,
        chrono::Utc::now().timestamp(),  // ← Forces pod recreation
        ctx.pack_name
    );
    
    let _ = nya.trigger("log", Payload::new(format!("Deploying {}...", ctx.pack_name))).await;
    
    let mut cmd = Command::new("ssh");
    cmd.args([
        "-i", &ssh_key,
        "-o", "StrictHostKeyChecking=no",
        &format!("{}@{}", user, host),
        &helm_cmd
    ])
    .stdin(Stdio::null())
    .stdout(Stdio::piped())
    .stderr(Stdio::piped());
    
    run_command(cmd, nya).await;
}

// Reuse this for all commands
async fn run_command(mut cmd: Command, nya: Nya) {
    // CRITICAL: Pass through SSH agent socket
    if let Ok(ssh_auth_sock) = env::var("SSH_AUTH_SOCK") {
      cmd.env("SSH_AUTH_SOCK", ssh_auth_sock);
    }

    let mut child = cmd.spawn().unwrap();
    
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    
    let mut out_reader = BufReader::new(stdout).lines();
    let mut err_reader = BufReader::new(stderr).lines();
    
    let out_task = tokio::spawn({
        let nya = nya.clone();
        async move {
            while let Ok(Some(line)) = out_reader.next_line().await {
                let _ = nya.trigger("log", Payload::new(line)).await;
            }
        }
    });
    
    let err_task = tokio::spawn({
        let nya = nya.clone();
        async move {
            while let Ok(Some(line)) = err_reader.next_line().await {
                let _ = nya.trigger("log", Payload::new(line)).await;
            }
        }
    });
    
    let _ = child.wait().await;
    let _ = out_task.await;
    let _ = err_task.await;
}