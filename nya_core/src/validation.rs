use std::collections::HashMap;
use crate::errors::NyaError;
use crate::errors::NyaError::{NyaPlanStepDoesNotExist, NyaPlanValidationFailure};
use crate::event_bus::{EventBus, NyaEventBus};
use crate::plan::NyaPlan;

pub fn validate_plans(plans: &HashMap<String, NyaPlan>) -> Result<(), NyaError>{
  let plan_iter = plans.iter();
  for plan in plan_iter {
    if plan.0.is_empty() {
      return Err(NyaPlanValidationFailure("Plans must not have an empty key".to_owned()));
    }

    if plan.1.steps.is_empty() {
      return Err(NyaPlanValidationFailure(format!("Plan steps must not be empty -> {0}", plan.0.as_str())));
    }
  }
  Ok(())
}

pub fn validate_run(cmd: &str, plan: &NyaPlan, bus: &NyaEventBus) -> Result<(), NyaError> {
  let plan_steps = &plan.steps;
  for step in plan_steps {
    let exists = bus.verify(step);
    if !exists {
      return Err(NyaPlanStepDoesNotExist {
        cmd: cmd.to_string(),
        step: step.to_string()
      })
    }
  }
  Ok(())
}
