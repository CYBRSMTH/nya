use crate::core::service::{Service, ServiceActions, handle_action};
use crate::cloud::base::build::cluster::{complete_cluster, register_node, setup_bind9, setup_helm, setup_tls, on_build_complete, on_finish};
use crate::cloud::base::build::control_plane::build_control_plane_action;
use crate::cloud::base::build::ingress::setup_ingress;
use crate::cloud::base::build::prebuild::{prebuild_action, run_prebuild_script};
pub(crate) mod prebuild;
pub(crate) mod control_plane;
pub(crate) mod cluster;
pub(crate) mod ingress; 


pub struct NyaBaseBuild;

impl Service for NyaBaseBuild {
  fn name(&self) -> String {"NyaBase".to_string()}
  fn register(&self) -> ServiceActions {
    vec![
      (String::from("onPreBuild"), handle_action(prebuild_action)),
      (String::from("runPreBuild"), handle_action(run_prebuild_script)),
      (String::from("onBuildControlPlane"), handle_action(build_control_plane_action)),
      (String::from("onCompleteCluster"), handle_action(complete_cluster)),
      (String::from("registerNode"), handle_action(register_node)),
      (String::from("setupBind9"), handle_action(setup_bind9)),
      (String::from("setupHelm"), handle_action(setup_helm)),
      (String::from("setupTLS"), handle_action(setup_tls)),
      (String::from("onClusterReady"), handle_action(setup_ingress)),
      (String::from("onBuildComplete"), handle_action(on_build_complete)),
      (String::from("onFinish"), handle_action(on_finish)),
    ]
  }
}