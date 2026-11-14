use std::collections::HashMap;
use serde_json::{Value, Map};
use crate::errors::NyaError;
use crate::errors::NyaError::NyaContextInvalidType;

#[derive(Clone, Debug)]
pub struct NyaContext {
  pub context: HashMap<String, Value>
}

impl NyaContext {
  pub fn new(configs: Vec<Value>) -> Result<NyaContext, NyaError> {
    let mut invalid_config: bool = false;
    let context: Map<String, Value> = configs.into_iter()
      .fold(Map::new(), | mut acc, config: Value| {
        if let Value::Object(map) = config {
          for (k, value) in map {
            acc.insert(k, value);
          }
        } else {
          invalid_config = true;
        }
        acc
      });

    if invalid_config {
      return Err(NyaContextInvalidType);
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
  use crate::errors::NyaError;

  #[test]
  fn get_nya_context_returns_context() -> Result<()> {
    let path = PathBuf::from("./tests/nya_test_config.json");
    let content = read_to_string(&path)
        .context(format!("Failed to read context file '{}'", path.display()))?;

    let json: Value = serde_json::from_str(&content)
        .context(format!("Failed to parse {}", path.display()))?;
    let nya_context = NyaContext::new(vec![json])?;
  
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
    let nya_context = NyaContext::new(vec![config_json, capsule_json])?;
  
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

  #[test]
  fn context_throws_an_error_when_config_is_not_an_object() -> Result<()> {
    let path = PathBuf::from("./tests/nya_context_invalid.json");
    let content = read_to_string(&path)
        .context(format!("Failed to read context file '{}'", path.display()))?;

    let json: Value = serde_json::from_str(&content)
        .context(format!("Failed to parse {}", path.display()))?;
    let nya_context = NyaContext::new(vec![json]);

    assert!(matches!(nya_context.unwrap_err(), NyaError::NyaContextInvalidType));
    Ok(())
  }
}