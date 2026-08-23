use crate::{payload::{Get, Payload}, service::{Service, ServiceActions, handle_action}, runtime::Nya};

pub struct NyaCoreServices;

impl Service for NyaCoreServices {
  fn name(&self) -> String {"NyaCoreServices".to_string()}
  fn register(&self) -> ServiceActions {
    vec![
      ("test".to_string(), handle_action(test_nya_service)),
      ("log".to_string(), handle_action(log))
    ]
  }
}

pub async fn log(_: Nya, payload: Payload) {
  println!("{}", payload.get::<String>().unwrap());
}

pub async fn test_nya_service(nya: Nya, payload: Payload) {
  let ctx_val = nya.get("test").await;
  let pay_val = payload.get::<&str>().unwrap();
  println!("Value from payload: {}", pay_val);
  println!("Value from context: {}", ctx_val.to_string());
  nya.trigger("log", Payload::new("test_log")).await;
}