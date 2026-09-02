use tokio::{sync::Mutex, task::JoinHandle};
use anyhow::Result;

pub struct TaskTracker {
    handles: Mutex<Vec<JoinHandle<Result<()>>>>,
}

impl TaskTracker {
    pub fn new() -> Self { Self { handles: Mutex::new(vec![]) } }
    pub async fn add(&self, handle: JoinHandle<Result<()>>) {
        self.handles.lock().await.push(handle);
    }
    pub async fn wait_all(&self) {
        let mut handles = self.handles.lock().await;
        for handle in handles.drain(..) {
            let _ = handle.await;
        }
    }
}