use std::sync::Arc;
use futures::{future::BoxFuture, FutureExt};
use crate::{payload::Payload, runtime::Nya};

pub type Action = Arc<dyn Fn(Nya, Payload) -> BoxFuture<'static, ()> + Send + Sync>;
pub type ServiceActions = Vec<(String, Action)>;

pub fn handle_action<F, Fut>(f: F) -> Action
where
    F: Fn(Nya, Payload) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = ()> + Send + 'static, {
    Arc::new(move |nya: Nya, payload: Payload| f(nya, payload).boxed())
}

pub trait Service: Send + Sync + 'static {
    fn name(&self) -> String;
    fn actions(&self) -> ServiceActions;
}

#[cfg(test)]
pub mod service_tests{
  use crate::{payload::Payload, service::Service};
  use anyhow::Result;
  use crate::test_utils::{get_test_nya, TestService};

  #[tokio::test]
  async fn can_create_service() {
    let svc = Box::new(TestService);
    let new_svc = &svc.actions();
    let new_svc_name = &svc.name();
    let value: usize = 2;
    assert_eq!(*new_svc_name, "Test Service".to_string());
    assert_eq!(value, new_svc.len());
  }

  #[tokio::test]
  async fn can_run_action() -> Result<()> {
    let test_nya = get_test_nya()?;
    {
      test_nya.trigger("event1", Payload::empty()).await;
    }
    tokio::task::yield_now().await;
    let value_json = test_nya.get("key1").await;
    let value = value_json.as_str().unwrap();
    assert_eq!(value, "value1");
    Ok(())
  }
}