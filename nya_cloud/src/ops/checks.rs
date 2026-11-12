use openssh::Session;

const IS_PYTHON3_INSTALLED: &str = "command -v python3 > /dev/null 2>&1";
const IS_DOCKER_INSTALLED: &str = "command -v docker > /dev/null 2>&1";
const IS_K3S_INSTALLED: &str = "command -v k3s > /dev/null 2>&1";
const IS_K3S_RUNNING: &str = "systemctl is-active --quiet k3s";
const IS_K3S_AGENT_RUNNING: &str = "systemctl is-active --quiet k3s-agent";
const IS_REGISTRY_RUNNING: &str = "docker ps --filter name=registry --filter status=running | grep -q registry";
const IS_MKCERT_INSTALLED: &str = "command -v mkcert > /dev/null 2>&1";
const IS_HELM_INSTALLED: &str = "command -v helm > /dev/null 2>&1";

pub enum CheckIf {
  Python3IsInstalled,
  DockerIsInstalled,
  K3sIsInstalled,
  K3sIsRunning,
  K3sAgentIsRunning,
  RegistryIsRunning,
  MkcertIsInstalled,
  HelmIsInstalled,
}

fn get_cmd(check: CheckIf) -> &'static str {
  match check {
    CheckIf::Python3IsInstalled => IS_PYTHON3_INSTALLED,
    CheckIf::DockerIsInstalled => IS_DOCKER_INSTALLED,
    CheckIf::K3sIsInstalled => IS_K3S_INSTALLED,
    CheckIf::K3sIsRunning => IS_K3S_RUNNING,
    CheckIf::K3sAgentIsRunning => IS_K3S_AGENT_RUNNING,
    CheckIf::RegistryIsRunning => IS_REGISTRY_RUNNING,
    CheckIf::MkcertIsInstalled => IS_MKCERT_INSTALLED,
    CheckIf::HelmIsInstalled => IS_HELM_INSTALLED,
  }
}

pub struct Check {}

impl Check {
  pub async fn run(check: CheckIf, session: &Session) -> bool {
    let command = get_cmd(check);
    match session.command("/bin/sh")
        .arg("-c")
        .arg(command)
        .output()
        .await
    {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
  }
}
  