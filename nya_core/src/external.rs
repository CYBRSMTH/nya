use crate::{core_services::NyaCoreServices, service::Service};

pub fn get_core_services() -> Vec<Box<dyn Service>> {
  vec![
    Box::new(NyaCoreServices),
  ]
}