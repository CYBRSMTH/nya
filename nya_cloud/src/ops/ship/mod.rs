use std::{env, path::PathBuf, process::Stdio};
use futures::future::join_all;
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use tokio::{io::{AsyncBufReadExt, BufReader}, process::Command};
use nya_core::{payload::Payload, runtime::Nya, service::{Service, ServiceActions, handle_action}};
use crate::ops::utils::prepare_base_context;

pub struct NyaShip;

impl Service for NyaShip {
  fn name(&self) -> String {"NyaShip".to_string()}
  fn actions(&self) -> ServiceActions {
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
  let capsule_path = nya.get("capsule_path").await;
  let full_capsule_path = capsule_path.as_str().unwrap_or("");
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
            nya.log(&line).await;
          }
      }
  });

  // Pump stderr
  let err_task = tokio::spawn({
      let nya = nya.clone();
      async move {
          while let Ok(Some(line)) = err_reader.next_line().await {
            nya.log(&line).await;
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
            if let Err(error) = push_image(&ctx.pack_image_name, nya.clone()).await {
                nya.log(&format!("Image upload failed: {error:#}")).await;
                return;
            }
            
            // Copy values
            copy_values(&ctx, nya.clone()).await;
            
            // Helm deploy
            helm_deploy(&ctx, nya.clone()).await;
        });
        deploy_tasks.push(deploy_task);
    }
    
    join_all(deploy_tasks).await;
}

async fn push_image(image_name: &str, nya: Nya) -> Result<()> {
    nya.log(&format!("Pushing {}...", image_name)).await;
    
    let mut cmd = Command::new("docker");
    cmd.args(["push", image_name])
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    
    run_command(cmd, nya).await.with_context(|| format!("could not push image {image_name}"))
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
    
    let _ = run_command(cmd, nya).await;
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
    
    nya.log(&format!("Deploying {}...", ctx.pack_name)).await;
    
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
    
    let _ = run_command(cmd, nya).await;
}

// Reuse this for all commands
async fn run_command(mut cmd: Command, nya: Nya) -> Result<()> {
    // CRITICAL: Pass through SSH agent socket
    if let Ok(ssh_auth_sock) = env::var("SSH_AUTH_SOCK") {
      cmd.env("SSH_AUTH_SOCK", ssh_auth_sock);
    }

    let mut child = cmd.spawn().context("could not start command")?;
    
    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();
    
    let mut out_reader = BufReader::new(stdout).lines();
    let mut err_reader = BufReader::new(stderr).lines();
    
    let out_task = tokio::spawn({
        let nya = nya.clone();
        async move {
            while let Ok(Some(line)) = out_reader.next_line().await {
                nya.log(&line).await;
            }
        }
    });
    
    let err_task = tokio::spawn({
        let nya = nya.clone();
        async move {
            let mut insecure_registry = false;
            while let Ok(Some(line)) = err_reader.next_line().await {
                insecure_registry |= line.contains("server gave HTTP response to HTTPS client");
                nya.log(&line).await;
            }
            insecure_registry
        }
    });
    
    let status = child.wait().await.context("could not wait for command")?;
    let _ = out_task.await;
    let insecure_registry = err_task.await?;
    if !status.success() {
        if insecure_registry {
            bail!("command failed with {status}; check Docker's insecure-registry setting for this HTTP registry");
        }
        bail!("command failed with {status}");
    }
    Ok(())
}
#[cfg(all(test, unix))]
mod tests {
    use super::*;

    fn test_nya() -> Nya {
        Nya::build("test", vec![serde_json::json!({"plans": {"test": {"steps": []}}})], vec![], Payload::empty()).unwrap()
    }

    fn shell(script: &str) -> Command {
        let mut command = Command::new("sh");
        command.args(["-c", script]).stdout(Stdio::piped()).stderr(Stdio::piped());
        command
    }

    #[tokio::test]
    async fn successful_command_returns_ok() {
        run_command(shell("printf uploaded; exit 0"), test_nya()).await.unwrap();
    }

    #[tokio::test]
    async fn failed_upload_does_not_continue_to_deployment() {
        if env::var_os("NYA_SHIP_TEST_WORKER").is_some() {
            let nya = test_nya();
            nya.set("pack_contexts", vec![PackContext {
                pack_name: "test".into(), pack_location: "/unused".into(), pack_image_name: "test-image".into(),
            }]).await;
            nya.set("nya.control_plane", serde_json::json!({"host": "unused", "user": "unused", "ssh_private_key_file": "unused"})).await;
            nya.set("nya.control_plane.vars", serde_json::json!({"domain_name": "unused", "secret_name": "unused"})).await;
            nya.set("nya.registry_host", "unused").await;
            deploy_capsule(nya, Payload::empty()).await;
            return;
        }

        // Change PATH only in a child test process, never in the parallel test runner.
        use std::os::unix::fs::PermissionsExt;
        let dir = env::temp_dir().join(format!("nya-ship-{}-{}", std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        std::fs::create_dir_all(&dir).unwrap();
        let trace = dir.join("trace");
        for (program, code) in [("docker", 1), ("scp", 0), ("ssh", 0)] {
            let path = dir.join(program);
            std::fs::write(&path, format!("#!/bin/sh\nprintf '%s\\n' '{program}' >> \"$NYA_SHIP_TRACE\"\nexit {code}\n")).unwrap();
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        let output = std::process::Command::new(env::current_exe().unwrap())
            .args(["--exact", "ops::ship::tests::failed_upload_does_not_continue_to_deployment", "--nocapture"])
            .env("NYA_SHIP_TEST_WORKER", "1").env("NYA_SHIP_TRACE", &trace)
            .env("PATH", &dir).output().unwrap();
        let calls = std::fs::read_to_string(&trace).unwrap();
        std::fs::remove_dir_all(dir).unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(calls, "docker\n", "copy/deploy must not run after a failed upload");
    }

    #[tokio::test]
    async fn http_registry_failure_mentions_insecure_registry_setting() {
        let error = run_command(shell("echo 'server gave HTTP response to HTTPS client' >&2; exit 1"), test_nya()).await.unwrap_err();
        assert!(error.to_string().contains("insecure-registry"));
    }

    #[tokio::test]
    async fn spawn_failure_is_returned() {
        let mut command = Command::new("/nonexistent/nya-test-docker");
        command.stdout(Stdio::piped()).stderr(Stdio::piped());
        assert!(run_command(command, test_nya()).await.is_err());
    }
}
