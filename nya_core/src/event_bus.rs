use std::collections::HashMap;
use std::sync::Arc;
use crate::errors::NyaError;
use crate::errors::NyaError::{NyaServiceActionsDuplicates, NyaServiceMissingActions};
use crate::payload::Payload;
use crate::service::{Action, Service};
use crate::runtime::Nya;

pub struct NyaEventBus {
  event_handlers: HashMap<String, Action>
}

impl NyaEventBus {
  pub fn new(reg: Vec<Box<dyn Service>>) -> Result<Self, NyaError> {
    let mut new_bus = Self {
      event_handlers: HashMap::new(),
    };

    let mut service_handlers = Vec::new();
    for service in reg.iter() {
      service_handlers.extend(service.actions());
    }
    
    if service_handlers.is_empty() {
      return Err(NyaServiceMissingActions);
    }
    
    for handler in service_handlers {
      let existing_handler =  &new_bus.event_handlers.get(&handler.0);
      if !existing_handler.is_none() {
        return Err(NyaServiceActionsDuplicates(handler.0.clone()));
      }
      new_bus.register(handler.0, handler.1);
    }
    Ok(new_bus)
  }
}

#[async_trait::async_trait]
pub trait EventBus: Send + Sync + 'static {
  fn register(&mut self, event: String, handler: Action);
  async fn emit(&self, nya: Nya, event: String, payload: Payload);
  fn verify(&self, event: &String) -> bool;
}

#[async_trait::async_trait]
impl EventBus for NyaEventBus {
  fn register(&mut self, event: String, handler: Action) {
    self.event_handlers.insert(event, handler);
  }
  
  async fn emit(&self, nya: Nya, event: String, payload: Payload) {
    let handler = self.event_handlers.get(&event).map(Arc::clone).unwrap();
    handler(nya, payload).await;
  }
  
  fn verify(&self, event: &String) -> bool {
    let handler = self.event_handlers.get(event);
    handler.is_some()
  }
}

#[cfg(test)]
mod event_bus_tests{
  use std::fs::read_to_string;
  use std::path::PathBuf;
  use serde_json::{from_value, Value};
  use anyhow::{Context, Result};

use crate::{
    {
      event_bus::{EventBus, NyaEventBus}, payload::Payload, service::
        {
          service_tests::TestService, Service
        }
    }, runtime::Nya};

  fn get_test_services() -> Vec<Box<dyn Service>> {
    vec![Box::new(TestService)]
  }

  #[tokio::test]
  async fn can_register_events() -> Result<()> {
    let event_bus = NyaEventBus::new(get_test_services())?;
    assert_eq!(event_bus.event_handlers.len(), 2);
    Ok(())
  }

  #[tokio::test]
  async fn event_bus_can_run_handlers_on_event() -> Result<()> {
    let path = PathBuf::from("./tests/nya_test_config.json");
    let content = read_to_string(&path)
        .context(format!("Failed to read context file '{}'", path.display()))?;

    let json: Value = serde_json::from_str(&content)
        .context(format!("Failed to parse {}", path.display()))?;
    let configs = vec![json];
    let event_bus = NyaEventBus::new(get_test_services())?;
    let test_nya = Nya::build("test_cmd", configs, vec![Box::new(TestService)])?;
    {
      event_bus.emit(test_nya.clone(), "test".to_string(), Payload::empty()).await;
    }
    tokio::task::yield_now().await;
    let ctx_val = test_nya.get("test_key").await;
    let value: String = from_value(ctx_val.clone())?;
    assert_eq!(value, "test_value".to_string());
    Ok(())
  }

  #[tokio::test]
  async fn event_bus_doesnt_run_if_theres_no_event() -> Result<()> {
    let path = PathBuf::from("./tests/nya_test_config.json");
    let content = read_to_string(&path)
        .context(format!("Failed to read context file '{}'", path.display()))?;

    let json: Value = serde_json::from_str(&content)
        .context(format!("Failed to parse {}", path.display()))?;
    let configs = vec![json];
    let event_bus = NyaEventBus::new(get_test_services())?;
    let test_nya = Nya::build("test_cmd", configs, vec![Box::new(TestService)])?;
    {
      event_bus.emit(test_nya.clone(), "fake_event".to_string(), Payload::empty()).await;
    }
    tokio::task::yield_now().await;
    let ctx_val = test_nya.get("test_key").await;
    assert_eq!(Value::Null, ctx_val);
    Ok(())
  }
}
