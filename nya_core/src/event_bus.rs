use std::collections::HashMap;
use std::sync::Arc;
use crate::payload::Payload;
use crate::service::Action;
use crate::runtime::Nya;

pub struct NyaEventBus {
  event_handlers: HashMap<String, Action>
}

impl NyaEventBus {
  pub fn new() -> Self {
    Self {
      event_handlers: HashMap::new(),
    }
  }
}

#[async_trait::async_trait]
pub trait EventBus: Send + Sync + 'static {
  fn on(&mut self, event: String, handler: Action);
  async fn emit(&self, nya: Nya, event: String, payload: Payload);
}

#[async_trait::async_trait]
impl EventBus for NyaEventBus {
  fn on(&mut self, event: String, handler: Action) {
    self.event_handlers.insert(event, handler);
  }
  
  async fn emit(&self, nya: Nya, event: String, payload: Payload) {
    if let Some(handler) = self.event_handlers.get(&event) {
      let nya_clone: Nya = nya.clone();
      let handler_clone = Arc::clone(handler);
      handler_clone(nya_clone, payload).await;
    } else {
      println!("No handler registered for event: {}", event);
    }
  }
}

#[cfg(test)]
mod event_bus_tests{
  use std::path::PathBuf;
  use serde_json::{from_value, Value};
  use anyhow::{Result};

use crate::{
    {
      event_bus::{EventBus, NyaEventBus}, payload::Payload, service::
        {
          service_tests::TestService, Service
        }
    }, runtime::Nya};

  #[tokio::test]
  async fn can_register_events() {
    let mut event_bus = NyaEventBus::new();
    let svc = Box::new(TestService);
    event_bus.on("test_event".to_string(), svc.register()[0].1.clone());
    assert_eq!(event_bus.event_handlers.len(), 1);
  }

  #[tokio::test]
  async fn event_bus_can_run_handlers_on_event() -> Result<()> {
    let configs = vec![PathBuf::from("./tests/nya_test_config.json")];
    let mut event_bus = NyaEventBus::new();
    let svc = Box::new(TestService);
    let handler= svc.register()[0].1.clone();
    let event_name = svc.register()[0].0.clone();
    let test_nya = Nya::build("test_cmd", configs, vec![Box::new(TestService)])?;
    {
      event_bus.on(event_name.clone(), handler);
      event_bus.emit(test_nya.clone(), event_name, Payload::empty()).await;
    }
    tokio::task::yield_now().await;
    let ctx_val = test_nya.get("test_key").await;
    let value: String = from_value(ctx_val.clone()).unwrap();
    assert_eq!(value, "test_value".to_string());
    Ok(())
  }

  #[tokio::test]
  async fn event_bus_doesnt_run_if_theres_no_event() -> Result<()> {
    let configs = vec![PathBuf::from("./tests/nya_test_config.json")];
    let mut event_bus = NyaEventBus::new();
    let svc = Box::new(TestService);
    let handler= svc.register()[0].1.clone();
    let event_name = svc.register()[0].0.clone();
    let test_nya = Nya::build("test_cmd", configs, vec![Box::new(TestService)])?;
    {
      event_bus.on(event_name.clone(), handler);
      event_bus.emit(test_nya.clone(), "fake_event".to_string(), Payload::empty()).await;
    }
    tokio::task::yield_now().await;
    let ctx_val = test_nya.get("test_key").await;
    assert_eq!(Value::Null, ctx_val);
    Ok(())
  }

}
