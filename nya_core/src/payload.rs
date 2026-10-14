use std::{any::{Any, type_name}, sync::Arc};
use anyhow::{Context, Result, anyhow};

#[derive(Clone, Default)]
pub struct Payload {
  value: Option<Arc<dyn Any + Send + Sync>>,
  type_name: Option<&'static str>,
}

impl Payload {
  pub fn empty() -> Self { Self::default() }
  pub fn new<T: Any + Send + Sync + 'static>(v: T) -> Self {
      Self {
          value: Some(Arc::new(v)),
          type_name: Some(type_name::<T>()),
      }
  }
  pub fn is_empty(&self) -> bool { self.value.is_none() }
}

#[derive(Debug)]
pub enum PayloadError {
  Empty,
  TypeMismatch { expected: &'static str, actual: &'static str },
  ArcStillShared
}

pub trait Get {
    fn get<T: Any>(&self) -> Result<&T, PayloadError>;
    fn into_arc<T: Any + Send + Sync + 'static>(self) -> Result<Arc<T>, PayloadError>
    where
        Self: Sized;
}

pub trait Take {
    fn take<T: Any + Send + Sync + 'static>(self) -> Result<T>
    where
        Self: Sized;
}

impl Get for Payload {
    fn get<T: Any>(&self) -> Result<&T, PayloadError> {
        let some = self.value.as_ref().ok_or(PayloadError::Empty)?;
        // Borrowed downcast
        (&**some as &dyn Any)
            .downcast_ref::<T>()
            .ok_or_else(|| PayloadError::TypeMismatch {
                expected: type_name::<T>(),
                actual: self.type_name.unwrap_or("unknown"),
            })
    }

    fn into_arc<T: Any + Send + Sync + 'static>(self) -> Result<Arc<T>, PayloadError> {
        let some = self.value.ok_or(PayloadError::Empty)?;
        // Owning downcast (requires owning the Arc)
        Arc::downcast::<T>(some).map_err(|_| PayloadError::TypeMismatch {
            expected: type_name::<T>(),
            actual: self.type_name.unwrap_or("unknown"),
        })
    }
}

impl Take for Payload {
    fn take<T: Any + Send + Sync + 'static>(self) -> Result<T> {
        let arc = self.value.context("Couldn't get the value from payload")?;
        // Downcast Arc<dyn Any> → Arc<T> first (T is Sized)
        let arc_t = arc.downcast::<T>()
            .map_err(|_| anyhow!("Could not get the value because the types mismatched"))?;
        // Now try_unwrap works because Arc<T> where T: Sized is fine

      Ok(Arc::try_unwrap(arc_t).map_err(|_| anyhow!("Could create the Arc because value is already shared"))?)
    }
}