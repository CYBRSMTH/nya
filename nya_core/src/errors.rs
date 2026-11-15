use thiserror::Error;

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum NyaError {
  #[error("One of the provided configs is not a valid type. Make sure your configs are json objects.")]
  NyaContextInvalidType,
  #[error("No plans were found. Please make sure your plans were added to your config.")]
  NyaPlansMissing,
  #[error("There was an issue parsing your plans: {0}")]
  NyaPlanParseFailed(#[source] serde_json::Error),
  #[error("Wasn't able to retrieve the plan for the command {0}")]
  NyaPlanInvalidCmd(String),
  #[error("{0}")]
  NyaPlanValidationFailure(String),
  #[error("Service actions must be unique. More than one action was found with the event name {0}")]
  NyaServiceActionsDuplicates(String),
  #[error("No services or actions were provided.")]
  NyaServiceMissingActions,
  #[error("No service action was found for the plan {cmd:?} and step {step:?}")]
  NyaPlanStepDoesNotExist{
    cmd:String,
    step: String
  },
}