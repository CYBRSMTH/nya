use std::collections::HashMap;
use serde_json::{Value, Map};
use anyhow::Result;
use crate::payload::{Payload, Take};

#[derive(Clone)]
pub struct NyaContext {
  pub context: HashMap<String, Value>
}

impl NyaContext {
  pub fn new(configs: Vec<Value>, initial_payload: Payload) -> Result<NyaContext> {
    let mut context: Map<String, Value> = configs
        .into_iter()
        .fold(Map::new(), | mut acc, config: Value| {
          if let Value::Object(map) = config {
            for (k, value) in map {
              acc.insert(k, value);
            }
          }
          acc
        });

    if !initial_payload.is_empty() {
      let payload_str = initial_payload.take::<String>()?;
      let payload_val = serde_json::from_str::<Value>(&payload_str)?;

      if let Value::Object(map) = payload_val {
        for (k, value) in map {
          context.insert(k, value);
        }
      } else if let Value::Array(arr) = payload_val {
        context.insert("initial_payload".to_string(), Value::Array(arr));
      }
    }

    Ok(Self {
      context: context.into_iter().collect()
    })
  }
}

#[cfg(test)]
mod context_tests {
  use std::fs::read_to_string;
  use std::path::PathBuf;
  use crate::context::NyaContext;
  use anyhow::{Context, Result};
  use serde_json::Value;
  use crate::payload::Payload;

  #[test]
  fn get_nya_context_returns_context() -> Result<()> {
    let path = PathBuf::from("./tests/nya_test_config.json");
    let content = read_to_string(&path)
        .context(format!("Failed to read context file '{}'", path.display()))?;

    let json: Value = serde_json::from_str(&content)
        .context(format!("Failed to parse {}", path.display()))?;
    let nya_context = NyaContext::new(vec![json], Payload::empty())?;
  
    let test_value = nya_context.context.get("test")
      .and_then(|v| v.as_str())
      .context("test1 not found or not a string")?;
    
    assert_eq!(test_value, "context_value");

    Ok(())
  }

  #[test]
  fn can_build_nya_context_from_multiple_locations() -> Result<()> {
    let config_path = PathBuf::from("./tests/nya_test_config.json");
    let capsule_path = PathBuf::from("./tests/nya_test_capsule.json");
    let config_content = read_to_string(&config_path)
        .context(format!("Failed to read context file '{}'", config_path.display()))?;

    let config_json: Value = serde_json::from_str(&config_content)
        .context(format!("Failed to parse {}", config_path.display()))?;

    let capsule_content = read_to_string(&capsule_path)
        .context(format!("Failed to read context file '{}'", capsule_path.display()))?;

    let capsule_json: Value = serde_json::from_str(&capsule_content)
        .context(format!("Failed to parse {}", capsule_path.display()))?;
    let nya_context = NyaContext::new(vec![config_json, capsule_json], Payload::empty())?;
  
    let test_value = nya_context.context.get("test")
      .and_then(|v| v.as_str())
      .context("test1 not found or not a string")?;

    let test_value2 = nya_context.context.get("capsule_name")
      .and_then(|v| v.as_str())
      .context("test1 not found or not a string")?;
    
    assert_eq!(test_value, "context_value");
    assert_eq!(test_value2, "my_capsule");

    Ok(())
  }
}