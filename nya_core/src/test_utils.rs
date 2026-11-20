use std::fs::read_to_string;
use std::path::PathBuf;
use anyhow::{Context, Result};
use serde_json::Value;
use crate::payload::Payload;
use crate::runtime::Nya;
use crate::service::{handle_action, Service, ServiceActions};

pub enum NyaTestConfigs {
  NyaConfig,
  NyaCapsule,
  NyaContextInvalid,
  NyaConfigNoPlans,
  NyaConfigPlanStepMissingAction,
  NyaConfigPlansInvalidJSON,
  NyaConfigPlansInvalidNoKey,
  NyaConfigPlansInvalidNoSteps
}

pub fn get_config(config: NyaTestConfigs) -> Result<Value> {
  let path_str = match config {
    NyaTestConfigs::NyaConfig => "./tests/positive/nya.json",
    NyaTestConfigs::NyaCapsule => "./tests/positive/nya_capsule.json",
    NyaTestConfigs::NyaContextInvalid => "./tests/negative/nya_context_invalid.json",
    NyaTestConfigs::NyaConfigNoPlans => "./tests/negative/nya_config_no_plans.json",
    NyaTestConfigs::NyaConfigPlanStepMissingAction => "./tests/negative/nya_config_plan_step_missing_action.json",
    NyaTestConfigs::NyaConfigPlansInvalidJSON => "./tests/negative/nya_config_plans_invalid_json.json",
    NyaTestConfigs::NyaConfigPlansInvalidNoKey => "./tests/negative/nya_config_plans_invalid_no_key.json",
    NyaTestConfigs::NyaConfigPlansInvalidNoSteps => "./tests/negative/nya_config_plans_invalid_no_steps.json"
  };

  let path = PathBuf::from(path_str);
  let content = read_to_string(&path)
      .context(format!("Failed to read context file '{}'", path.display()))?;
  serde_json::from_str(&content)
      .context(format!("Failed to parse {}", path.display()))
}

pub(crate) async fn test_fn(nya: Nya, _: Payload) {
  nya.set("key1", Value::String("value1".to_string())).await;
}
pub(crate) async fn test_fn2(nya: Nya, _: Payload) {
  nya.set("key2", Value::String("value2".to_string())).await;
}

pub(crate) struct TestService;
pub(crate) struct EmptyService;
pub(crate) struct DuplicateService;

impl Service for TestService {
  fn name(&self) -> String {"Test Service".to_string()}
  fn actions(&self) -> ServiceActions {
    vec![
      ("event1".to_string(), handle_action(test_fn)),
      ("event2".to_string(), handle_action(test_fn2))
    ]
  }
}

impl Service for EmptyService {
  fn name(&self) -> String {"Empty Service".to_string()}
  fn actions(&self) -> ServiceActions {
    vec![]
  }
}

impl Service for DuplicateService {
  fn name(&self) -> String {"Duplicate Service".to_string()}
  fn actions(&self) -> ServiceActions {
    vec![
      ("event1".to_string(), handle_action(test_fn)),
      ("event1".to_string(), handle_action(test_fn2)),
    ]
  }
}

pub(crate) fn get_test_nya(configs: Vec<NyaTestConfigs>) -> Result<Nya> {
  let configs = configs
      .into_iter()
      .map(get_config)
      .collect::<Result<Vec<Value>>>()?;
  Ok(Nya::build("cmd1", configs, vec![Box::new(TestService)])?)
}