use std::collections::HashMap;
use serde_json::{Value, Map};
use crate::errors::NyaError;
use crate::errors::NyaError::ContextInvalidType;

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
      return Err(ContextInvalidType);
    }

    Ok(Self {
      context: context.into_iter().collect()
    })
  }
}

#[cfg(test)]
mod context_tests {
  use crate::context::NyaContext;
  use anyhow::{Context, Result};
  use crate::errors::NyaError;
  use crate::test_utils::{get_config, NyaTestConfigs};

  #[test]
  fn get_nya_context_returns_context() -> Result<()> {
    let config = get_config(NyaTestConfigs::NyaConfig)?;
    let nya_context = NyaContext::new(vec![config])?;
    let test_value = nya_context.context.get("context_key1")
      .and_then(|v| v.as_str())
      .context("test1 not found or not a string")?;
    
    assert_eq!(test_value, "context_value1");

    Ok(())
  }

  #[test]
  fn can_build_nya_context_from_multiple_locations() -> Result<()> {
    let config_val = get_config(NyaTestConfigs::NyaConfig)?;
    let capsule_val = get_config(NyaTestConfigs::NyaCapsule)?;
    let nya_context = NyaContext::new(vec![config_val, capsule_val])?;
  
    let test_value = nya_context.context.get("context_key1")
      .and_then(|v| v.as_str())
      .context("test1 not found or not a string")?;
    let test_value2 = nya_context.context.get("capsule_name")
      .and_then(|v| v.as_str())
      .context("test1 not found or not a string")?;
    
    assert_eq!(test_value, "context_value1");
    assert_eq!(test_value2, "my_capsule");

    Ok(())
  }

  #[test]
  fn context_throws_an_error_when_config_is_not_an_object() -> Result<()> {
    let config = get_config(NyaTestConfigs::NyaContextInvalid)?;
    let nya_context = NyaContext::new(vec![config]);
    assert!(matches!(nya_context.unwrap_err(), NyaError::ContextInvalidType));
    Ok(())
  }

  #[test]
  fn context_throws_an_error_when_config_is_mixed_with_invalid_config() -> Result<()> {
    let config1 = get_config(NyaTestConfigs::NyaConfig)?;
    let config2 = get_config(NyaTestConfigs::NyaContextInvalid)?;
    let nya_context = NyaContext::new(vec![config1, config2]);
    assert!(matches!(nya_context.unwrap_err(), NyaError::ContextInvalidType));
    Ok(())
  }
}