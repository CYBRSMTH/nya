use nya_core::service::Service;
use crate::ops::base::build::NyaBaseBuild;
use crate::ops::base::destroy::NyaBaseDestroy;
use crate::ops::ship::NyaShip;

pub mod base;
pub mod ship;
mod utils;
pub(crate) mod types;
pub(crate) mod checks;

pub fn get_cloud_services() -> Vec<Box<dyn Service>> {
  vec![
    Box::new(NyaBaseBuild),
    Box::new(NyaBaseDestroy),
    Box::new(NyaShip)
  ]
}