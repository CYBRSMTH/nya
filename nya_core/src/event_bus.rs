use std::collections::HashMap;
use std::sync::Arc;
use crate::errors::NyaError;
use crate::errors::NyaError::{ServiceActionNotRegistered, ServiceActionsDuplicates, ServiceMissingActions};
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
      return Err(ServiceMissingActions);
    }

    for handler in service_handlers {
      let existing_handler =  &new_bus.event_handlers.get(&handler.0);
      if !existing_handler.is_none() {
        return Err(ServiceActionsDuplicates(handler.0.clone()));
      }
      new_bus.register(handler.0, handler.1);
    }
    Ok(new_bus)
  }
}

#[async_trait::async_trait]
pub trait EventBus: Send + Sync + 'static {
  fn register(&mut self, event: String, handler: Action);
  async fn emit(&self, nya: Nya, event: String, payload: Payload) -> Result<(), NyaError>;
  fn verify(&self, event: &String) -> bool;
}

#[async_trait::async_trait]
impl EventBus for NyaEventBus {
  fn register(&mut self, event: String, handler: Action) {
    self.event_handlers.insert(event, handler);
  }
  
  async fn emit(&self, nya: Nya, event: String, payload: Payload) -> Result<(), NyaError> {
    if let Some(handler) = self.event_handlers.get(&event).map(Arc::clone) {
      handler(nya, payload).await;
    } else {
      return Err(ServiceActionNotRegistered(event))
    }
    Ok(())
  }

  fn verify(&self, event: &String) -> bool {
    let handler = self.event_handlers.get(event);
    handler.is_some()
  }
}

#[cfg(test)]
mod event_bus_tests{
  use serde_json::from_value;
  use anyhow::Result;
  use crate::{{event_bus::{EventBus, NyaEventBus}, payload::Payload, service::Service}, runtime::Nya};
  use crate::errors::NyaError;
  use crate::test_utils::{get_config, DuplicateService, EmptyService, NyaTestConfigs, TestService};

  fn get_test_services() -> Vec<Box<dyn Service>> {
    vec![Box::new(TestService)]
  }
  fn get_empty_service() -> Vec<Box<dyn Service>> {
    vec![Box::new(EmptyService)]
  }
  fn get_dup_service() -> Vec<Box<dyn Service>> {vec![Box::new(DuplicateService)]}

  #[tokio::test]
  async fn can_register_events() -> Result<()> {
    let event_bus = NyaEventBus::new(get_test_services())?;
    assert_eq!(event_bus.event_handlers.len(), 2);
    Ok(())
  }

  #[tokio::test]
  async fn event_bus_can_run_handlers_on_event() -> Result<()> {
    let config = get_config(NyaTestConfigs::NyaConfig)?;
    let configs = vec![config];
    let event_bus = NyaEventBus::new(get_test_services())?;
    let test_nya = Nya::build("cmd1", configs, vec![Box::new(TestService)])?;
    {
      event_bus.emit(test_nya.clone(), "event1".to_string(), Payload::empty()).await?;
    }
    tokio::task::yield_now().await;
    let ctx_val = test_nya.get("key1").await;
    let value: String = from_value(ctx_val.clone())?;
    assert_eq!(value, "value1".to_string());
    Ok(())
  }

  #[tokio::test]
  async fn event_bus_doesnt_run_if_theres_no_event() -> Result<()> {
    let config = get_config(NyaTestConfigs::NyaConfig)?;
    let configs = vec![config];
    let event_bus = NyaEventBus::new(get_test_services())?;
    let test_nya = Nya::build("cmd1", configs, vec![Box::new(TestService)])?;
    let emit_result ;
    {
      emit_result = event_bus.emit(test_nya.clone(), "fake_event".to_string(), Payload::empty()).await;
    }
    tokio::task::yield_now().await;
    assert!(matches!( emit_result.unwrap_err(), NyaError::ServiceActionNotRegistered(_)));
    Ok(())
  }

  #[tokio::test]
  async fn event_bus_returns_error_when_service_is_empty() -> Result<()> {
    let bus_result = NyaEventBus::new(get_empty_service());
    assert!(matches!(bus_result, Err(NyaError::ServiceMissingActions)));
    Ok(())
  }

  #[tokio::test]
  async fn event_bus_returns_error_when_service_has_duplicate_actions() -> Result<()> {
    let bus_result = NyaEventBus::new(get_dup_service());
    assert!(matches!(bus_result, Err(NyaError::ServiceActionsDuplicates(_))));
    Ok(())
  }
}
