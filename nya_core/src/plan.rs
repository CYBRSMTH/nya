use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::context::NyaContext;
use anyhow::{Result, bail};

type NyaPlans = HashMap<String, NyaPlan>;
type NyaPlanSteps = Vec<String>;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct NyaPlan {
    pub steps: NyaPlanSteps,
}

impl NyaPlan {
    pub fn new(cmd: &str, ctx: NyaContext) -> Result<Self> {
       let plan = get_plan(cmd, ctx)?;
        Ok(Self {
            steps: plan,
          }
        )
    }
    
    // TODO: create fn to return plans, will have to set all plans in struct first
    // pub fn list_schemas(&self) -> Vec<&String> {
    //     self.plans.keys().collect()
    // }
}

fn get_plan(cmd: &str, ctx: NyaContext) -> Result<NyaPlanSteps> {
  let ctx_plans_json = ctx.context.get("plans");
  let plans = match ctx_plans_json {
    Some(json) => serde_json::from_value::<NyaPlans>(json.clone())?,
    None => {
      bail!("No plans were found. Please verify plans were added to your config".to_string());
    }
  };
  if let Some(plan) = plans.get(cmd).clone() {
      return Ok(plan.clone().steps);
  }
  bail!("Get plan(): wasn't able to successfully retrieve plan".to_string());
}

#[cfg(test)]
mod schema_tests {
  use std::path::PathBuf;
  use crate::context::NyaContext;
  use crate::plan::NyaPlan;
  use anyhow::Result;

  #[test]
    fn can_get_plan() -> Result<()> {
      let path = PathBuf::from("./tests/nya_test_config.json");
      let nya_context = NyaContext::new(vec![path])?;

      let found = NyaPlan::new("test_cmd", nya_context)?;
      let steps_len: usize = 2;
      assert_eq!(found.steps.len(), steps_len);
      Ok(())
    }
  
    #[test]
    fn returns_error_for_nonexistent_plan() -> Result<()> {
      let path = PathBuf::from("./tests/nya_test_config.json");
      let nya_context = NyaContext::new(vec![path])?;
      let result = NyaPlan::new("nonexistent", nya_context);

      assert!(result.is_err());
      Ok(())
    }
}