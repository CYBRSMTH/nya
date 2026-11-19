use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::context::NyaContext;
use crate::errors::NyaError;
use crate::errors::NyaError::{PlanInvalidCmd, PlansMissing, PlanParseFailed};
use crate::validation::validate_plans;

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
      .ok_or_else(|| PlansMissing)?;
  let mut plans = NyaPlans::deserialize(ctx_plans_value)
      .map_err(|e| PlanParseFailed(e))?;
  validate_plans(&plans)?;
  let plan = plans.remove(cmd)
      .ok_or_else(|| PlanInvalidCmd(cmd.to_string()))?;
  Ok(plan.steps)
}

#[cfg(test)]
mod plan_tests {
  use crate::context::NyaContext;
  use crate::plan::NyaPlan;
  use anyhow::Result;
  use crate::errors::NyaError;
  use crate::test_utils::{get_config, NyaTestConfigs};

  #[test]
    fn can_get_plan() -> Result<()> {
      let config = get_config(NyaTestConfigs::NyaConfig)?;
      let nya_context = NyaContext::new(vec![config])?;
      let found = NyaPlan::new("cmd1", nya_context)?;
      let steps_len: usize = 1;
      assert_eq!(found.steps.len(), steps_len);
      Ok(())
    }
  
    #[test]
    fn returns_error_for_nonexistent_plan() -> Result<()> {
      let config = get_config(NyaTestConfigs::NyaConfig)?;
      let nya_context = NyaContext::new(vec![config])?;
      let result = NyaPlan::new("nonexistent", nya_context);
      assert!(matches!(result.unwrap_err(), NyaError::PlanInvalidCmd(_)));
      Ok(())
    }

  #[test]
  fn returns_error_if_plans_dont_exist() -> Result<()> {
    let config = get_config(NyaTestConfigs::NyaConfigNoPlans)?;
    let nya_context = NyaContext::new(vec![config])?;
    let result = NyaPlan::new("cmd1", nya_context);
    assert!(matches!(result.unwrap_err(), NyaError::PlansMissing));
    Ok(())
  }

  #[test]
  fn returns_error_when_json_parsing_fails() -> Result<()> {
    let config = get_config(NyaTestConfigs::NyaConfigPlansInvalidJSON)?;
    let nya_context = NyaContext::new(vec![config])?;
    let result = NyaPlan::new("cmd1", nya_context);
    assert!(matches!(result.unwrap_err(), NyaError::PlanParseFailed(_)));
    Ok(())
  }

  #[test]
  fn returns_error_when_plan_key_is_missing() -> Result<()> {
    let config = get_config(NyaTestConfigs::NyaConfigPlansInvalidNoKey)?;
    let nya_context = NyaContext::new(vec![config])?;
    let result = NyaPlan::new("test_cmd", nya_context);
    assert!(matches!(result.unwrap_err(), NyaError::PlanValidationFailure(_)));
    Ok(())
  }

  #[test]
  fn returns_error_when_plan_steps_are_missing() -> Result<()> {
    let config = get_config(NyaTestConfigs::NyaConfigPlansInvalidNoSteps)?;
    let nya_context = NyaContext::new(vec![config])?;
    let result = NyaPlan::new("test_cmd", nya_context);
    assert!(matches!(result.unwrap_err(), NyaError::PlanValidationFailure(_)));
    Ok(())
  }

}