use tokio::{sync::Mutex, task::JoinHandle};

pub struct TaskTracker {
    handles: Mutex<Vec<JoinHandle<()>>>
}

impl TaskTracker {
    pub fn new() -> Self { Self { handles: Mutex::new(vec![]) } }
    pub async fn add(&self, handle: JoinHandle<()>) {
        self.handles.lock().await.push(handle);
    }
    pub async fn wait_all(&self) {
        let mut handles = self.handles.lock().await;
        for handle in handles.drain(..) {
            let _ = handle.await;
        }
    }
}