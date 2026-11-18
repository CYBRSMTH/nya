use std::fs::read_to_string;
use std::path::PathBuf;
use anyhow::{Context, Result};
use serde_json::Value;
use crate::payload::Payload;
use crate::runtime::Nya;
use crate::service::{handle_action, Service, ServiceActions};

pub(crate) async fn test_fn(nya: Nya, _: Payload) {
  nya.set("key1", Value::String("value1".to_string())).await;
}
pub(crate) async fn test_fn2(nya: Nya, _: Payload) {
  nya.set("key2", Value::String("value2".to_string())).await;
}

pub(crate) struct TestService;
impl Service for TestService {
  fn name(&self) -> String {"Test Service".to_string()}
  fn actions(&self) -> ServiceActions {
    vec![
      ("event1".to_string(), handle_action(test_fn)),
      ("event2".to_string(), handle_action(test_fn2))
    ]
  }
}

pub(crate) fn get_test_nya() -> Result<Nya> {
  let path = PathBuf::from("./tests/positive/nya.json");
  let content = read_to_string(&path)
      .context(format!("Failed to read context file '{}'", path.display()))?;
  let json: Value = serde_json::from_str(&content)
      .context(format!("Failed to parse {}", path.display()))?;
  let configs = vec![json];
  Ok(Nya::build("cmd1", configs, vec![Box::new(TestService)])?)
}