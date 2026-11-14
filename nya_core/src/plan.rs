use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::context::NyaContext;
use crate::errors::NyaError;
use crate::errors::NyaError::{NyaPlanInvalidCmd, NyaPlansMissing, NyaPlanParseFailed};

type NyaPlanSteps = Vec<String>;
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NyaPlan {
    pub steps: NyaPlanSteps,
}

type NyaPlans = HashMap<String, NyaPlan>;

impl NyaPlan {
    pub fn new(cmd: &str, ctx: NyaContext) -> Result<NyaPlan, NyaError> {
       let plan = get_plan(cmd, ctx)?;
        Ok(Self {
            steps: plan,
          }
        )
    }
}

fn get_plan(cmd: &str, ctx: NyaContext) -> Result<NyaPlanSteps, NyaError> {
  let ctx_plans_value = ctx.context.get("plans")
      .ok_or_else(|| NyaPlansMissing)?;
  let mut plans = NyaPlans::deserialize(ctx_plans_value)
      .map_err(|e| NyaPlanParseFailed(e))?;
  let plan = plans.remove(cmd)
      .ok_or_else(|| NyaPlanInvalidCmd(cmd.to_string()))?;
  Ok(plan.steps)
}

#[cfg(test)]
mod plan_tests {
  use std::fs::read_to_string;
  use std::path::PathBuf;
  use crate::context::NyaContext;
  use crate::plan::NyaPlan;
  use anyhow::{Context, Result};
  use serde_json::Value;
  use crate::errors::NyaError;

  #[test]
    fn can_get_plan() -> Result<()> {
    let path = PathBuf::from("./tests/nya_test_config.json");
    let content = read_to_string(&path)
        .context(format!("Failed to read context file '{}'", path.display()))?;

    let json: Value = serde_json::from_str(&content)
        .context(format!("Failed to parse {}", path.display()))?;
      let nya_context = NyaContext::new(vec![json])?;

      let found = NyaPlan::new("test_cmd", nya_context)?;
      let steps_len: usize = 2;
      assert_eq!(found.steps.len(), steps_len);
      Ok(())
    }
  
    #[test]
    fn returns_error_for_nonexistent_plan() -> Result<()> {
      let path = PathBuf::from("./tests/nya_test_config.json");
      let content = read_to_string(&path)
          .context(format!("Failed to read context file '{}'", path.display()))?;

      let json: Value = serde_json::from_str(&content)
          .context(format!("Failed to parse {}", path.display()))?;
      let nya_context = NyaContext::new(vec![json])?;
      let result = NyaPlan::new("nonexistent", nya_context);

      assert!(result.is_err());
      assert!(matches!(result.unwrap_err(), NyaError::NyaPlanInvalidCmd(_)));
      Ok(())
    }

  #[test]
  fn returns_error_if_plans_dont_exist() -> Result<()> {
    let path = PathBuf::from("./tests/nya_test_no_plans.json");
    let content = read_to_string(&path)
        .context(format!("Failed to read context file '{}'", path.display()))?;

    let json: Value = serde_json::from_str(&content)
        .context(format!("Failed to parse {}", path.display()))?;
    let nya_context = NyaContext::new(vec![json])?;
    let result = NyaPlan::new("test_cmd", nya_context);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), NyaError::NyaPlansMissing));
    Ok(())
  }

  #[test]
  fn returns_error_when_json_parsing_fails() -> Result<()> {
    let path = PathBuf::from("./tests/nya_test_plans_invalid_json.json");
    let content = read_to_string(&path)
        .context(format!("Failed to read context file '{}'", path.display()))?;

    let json: Value = serde_json::from_str(&content)
        .context(format!("Failed to parse {}", path.display()))?;
    let nya_context = NyaContext::new(vec![json])?;
    let result = NyaPlan::new("test_cmd", nya_context);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), NyaError::NyaPlanParseFailed(_)));
    Ok(())
  }
}