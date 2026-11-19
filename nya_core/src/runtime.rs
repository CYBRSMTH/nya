use std::sync::Arc;
use serde::Serialize;
use serde_json::Value;
use tokio::{sync::Mutex, task::JoinHandle};
use crate::{context::NyaContext, event_bus::{EventBus, NyaEventBus}, payload::Payload, plan::NyaPlan, service::Service, task_tracker::TaskTracker};
use crate::errors::NyaError;
use crate::validation::validate_run;

struct NyaInternals {
  context: Arc<Mutex<NyaContext>>,
  plan: NyaPlan,
  bus: Arc<NyaEventBus>,
  task_tracker: TaskTracker,
}

#[derive(Clone)]
pub struct Nya {
  internals: Arc<NyaInternals>
}

impl Nya {
  pub async fn run(plan: &str, configs: Vec<Value>, services: Vec<Box<dyn Service>>, initial_payload: Payload) -> Result<(), NyaError> {
    let nya = Nya::build(&plan, configs, services)?;
    nya.execute(initial_payload).await?;
    Ok(())
  }

  pub fn build(cmd: &str, configs: Vec<Value>, services: Vec<Box<dyn Service>>) -> Result<Self, NyaError> {
    let nya_event_bus = NyaEventBus::new(services)?;
    let ctx = NyaContext::new(configs)?;
    let plan = NyaPlan::new(&cmd, ctx.clone())?;
    let internals = NyaInternals {
      context: Arc::new(Mutex::new(ctx)),
      plan,
      bus: Arc::new(nya_event_bus),
      task_tracker: TaskTracker::new(),
    };

    validate_run(
      &cmd,
      &internals.plan,
      &internals.bus
    )?;

    Ok(Self {
      internals: Arc::new(internals)
    })
  }

  pub async fn execute(&self, initial_payload: Payload) -> Result<(), NyaError> {
    let mut payload = initial_payload;
    for step in self.internals.plan.steps.iter() {
      self.internals.bus.emit(self.clone(), step.clone(), std::mem::take(&mut payload)).await?;
      self.internals.task_tracker.wait_all().await;
    }
    Ok(())
  }

  pub async fn get(&self, key: &str) -> Value {
    let ctx = self.internals.context.lock().await;
    if let Some(item) = ctx.context.get(key) {
      return item.clone()
    }
    return Value::Null;
  }

  pub async fn set<T: Serialize>(&self, key: &str, value: T) {
    let mut ctx = self.internals.context.lock().await;
    if let Ok(json_value) = serde_json::to_value::<T>(value) {
      ctx.context.insert(key.to_string(), json_value);
    }
  }

  pub async fn trigger(&self, event: &str, payload: Payload) {
    let nya = self.clone();
    let event_name = event.to_string();
    let handle: JoinHandle<()> = tokio::spawn(async move {
      if let Err(e) = nya.internals.bus.emit(nya.clone(), event_name, payload).await {
        nya.log(&e.to_string()).await;
      }
    });
    self.internals.task_tracker.add(handle).await;
  }

  pub async fn trigger_all(&self, triggers: Vec<(&str, Payload)>) {
    for (event, payload) in triggers {
      self.trigger(event, payload).await;
    }
  }

  pub async fn log(&self, log: &str) {
    println!("{}", log);
  }
}

#[cfg(test)]
mod nya_tests {
  use crate::{payload::Payload, runtime::Nya};
  use anyhow::Result;
  use crate::test_utils::{get_config, NyaTestConfigs, TestService};

  #[test]
  fn can_build_nya() -> Result<()> {
    let config = get_config(NyaTestConfigs::NyaConfig)?;
    let _ = Nya::build("cmd1", vec![config],vec![Box::new(TestService)])?;
    Ok(())
  }

  #[tokio::test]
  async fn can_run_nya_schema() -> Result<()> {
    let config = get_config(NyaTestConfigs::NyaConfig)?;
    let nya = Nya::build("cmd1", vec![config], vec![Box::new(TestService)])?;
    {
      nya.execute(Payload::empty()).await?;
    }
    tokio::task::yield_now().await;
    let ctx = nya.internals.context.lock().await;
    let val1 = ctx.context.get("key1").unwrap().as_str().unwrap();
    assert_eq!("value1", val1);
    Ok(())
  }

#[tokio::test]
  async fn can_get_value_from_nya() -> Result<()>{
  let config = get_config(NyaTestConfigs::NyaConfig)?;
  let nya = Nya::build("cmd1", vec![config], vec![Box::new(TestService)])?;
  {
    nya.execute(Payload::empty()).await?;
  }
  tokio::task::yield_now().await;
  let nya_val = &nya.get("key1").await;
  assert_eq!("value1", nya_val.as_str().unwrap());
  Ok(())
  }

  #[tokio::test]
  async fn can_set_value_on_nya() -> Result<()>{
    let config = get_config(NyaTestConfigs::NyaConfig)?;
    let nya = Nya::build("cmd1", vec![config], vec![Box::new(TestService)])?;
    let _ = &nya.set("test_key", "test_value").await;
    let nya_val = &nya.get("test_key").await;
    let val1 = nya_val.as_str().unwrap();
    assert_eq!("test_value", val1);
    Ok(())
  }

  #[tokio::test]
  async fn can_trigger_nya_event() -> Result<()>{
    let config = get_config(NyaTestConfigs::NyaConfig)?;
    let nya = Nya::build("cmd1", vec![config], vec![Box::new(TestService)])?;
    {
      nya.trigger("event1", Payload::empty()).await;
    }
    tokio::task::yield_now().await;
    let ctx = nya.internals.context.lock().await;
    let val1 = ctx.context.get("key1").unwrap().as_str().unwrap();
    assert_eq!("value1", val1);
    Ok(())
  }
}