use std::sync::Arc;
use futures::{future::BoxFuture, FutureExt};
use crate::{payload::Payload, runtime::Nya};

pub type Action = Arc<dyn Fn(Nya, Payload) -> BoxFuture<'static, ()> + Send + Sync>;
pub type ServiceActions = Vec<(String, Action)>;

pub fn handle_action<F, Fut>(f: F) -> Action
where
    F: Fn(Nya, Payload) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static, {
    Arc::new(move |nya: Nya, payload: Payload| f(nya, payload).boxed())
}

pub trait Service: Send + Sync + 'static {
    fn name(&self) -> String;
    fn register(&self) -> ServiceActions;
}

#[cfg(test)]
pub mod service_tests{
  use std::fs::read_to_string;
  use crate::{payload::Payload, service::{handle_action, Service, Action, ServiceActions}, runtime::Nya};
  use anyhow::{Context, Result};
  use serde_json::Value;

  pub async fn test_fn(nya: Nya, _: Payload) {
    nya.set("test_key", serde_json::Value::String("test_value".to_string())).await;
  }
  pub async fn test_fn2(nya: Nya, _: Payload) {
    nya.set("test_key2", serde_json::Value::String("test_value2".to_string())).await;
  }

  pub struct TestService;
  impl Service for TestService {
    fn name(&self) -> String { "Test Service".to_string()}
    fn register(&self) -> ServiceActions {
        vec![
          ("test".to_string(), handle_action(test_fn)),
          ("test2".to_string(), handle_action(test_fn2))
        ]
    }
  }
  

  #[tokio::test]
  async fn can_create_action() -> Result<()> {
    use std::path::PathBuf;
    let path = PathBuf::from("./tests/nya_test_config.json");
    let content = read_to_string(&path)
        .context(format!("Failed to read context file '{}'", path.display()))?;

    let json: Value = serde_json::from_str(&content)
        .context(format!("Failed to parse {}", path.display()))?;
    let configs = vec![json];
    let new_svc_fn: Action = handle_action(test_fn);
    let test_nya = Nya::build("test_cmd", configs, vec![Box::new(TestService)], Payload::empty())?;
    new_svc_fn(test_nya.clone(), Payload::empty()).await;
    let value_json = test_nya.get("test_key").await;
    let value = value_json.as_str().unwrap();
    assert_eq!(value, "test_value");
    Ok(())
  }

  #[tokio::test]
  async fn can_create_service() {
    let svc = Box::new(TestService);
    let new_svc = &svc.register();
    let new_svc_name = &svc.name();
    let value: usize = 2;
    assert_eq!(*new_svc_name, "Test Service".to_string());
    assert_eq!(value, new_svc.len());
  }
}