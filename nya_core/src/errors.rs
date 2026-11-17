use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum NyaError {
  #[error("One of the provided configs is not a valid type. Make sure your configs are json objects.")]
  ContextInvalidType,
  #[error("No plans were found. Please make sure your plans were added to your config.")]
  PlansMissing,
  #[error("There was an issue parsing your plans: {0}")]
  PlanParseFailed(#[source] serde_json::Error),
  #[error("Wasn't able to retrieve the plan for the command {0}")]
  PlanInvalidCmd(String),
  #[error("{0}")]
  PlanValidationFailure(String),
  #[error("Service actions must be unique. More than one action was found with the event name {0}")]
  ServiceActionsDuplicates(String),
  #[error("No services or actions were provided.")]
  ServiceMissingActions,
  #[error("No service action was found for the plan {cmd:?} and step {step:?}")]
  PlanStepDoesNotExist{
    cmd:String,
    step: String
  },
  #[error("Nya received an event ({0}) that did not return a registered service action.")]
  ServiceActionNotRegistered(String),
  #[error("There was an issue with the provided Payload: {0}")]
  NyaPayloadError(String),
}