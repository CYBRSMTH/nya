use std::{collections::HashMap, fs::read_to_string};
use std::path::PathBuf;
use serde_json::{Value, Map};
use anyhow::{Context, Result};
use crate::payload::{Payload, Take};

#[derive(Clone)]
pub struct NyaContext {
  pub context: HashMap<String, Value>
}

impl NyaContext {
  pub fn new(configs: Vec<PathBuf>, initial_payload: Payload) -> Result<NyaContext> {
    let mut context: Map<String, Value> = Map::new();

    for path in configs.iter() {
      let content = read_to_string(&path)
          .context(format!("Failed to read context file '{}'", path.display()))?;

      let json: Value = serde_json::from_str(&content)
          .context(format!("Failed to parse {}", path.display()))?;

      if let Value::Object(map) = json {
        for (k, value) in map {
          context.insert(k, value);
        }
      }
    }

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
  use std::path::PathBuf;
  use crate::context::NyaContext;
  use anyhow::{Context, Result};
  use crate::payload::Payload;

  #[test]
  fn get_nya_context_returns_context() -> Result<()> {
    let path = PathBuf::from("./tests/nya_test_config.json");
    let nya_context = NyaContext::new(vec![path], Payload::empty())?;
  
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
    let nya_context = NyaContext::new(vec![config_path, capsule_path], Payload::empty())?;
  
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