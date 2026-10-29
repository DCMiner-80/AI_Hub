#[allow(dead_code)]
pub enum UiMessage { 
    Result { id: String, resp: ai_hub_core::ModelResponse } 
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct TaskSender {
    pub tx: tokio::sync::mpsc::Sender<String>,
}

/// The asynchronous background worker thread event loop
/// This listens to incoming UI task strings and processes them off the main thread.
pub async fn run_worker_loop(
    mut task_rx: tokio::sync::mpsc::Receiver<String>,
    _ui_tx: tokio::sync::mpsc::Sender<UiMessage>,
) {
    // Process messages as long as the UI task channel remains open
    while let Some(task_command) = task_rx.recv().await {
        // Log incoming background execution requests
        println!("[Worker] Received background task: {}", task_command);
        
        // TODO: Map task strings to ai_hub_core downloaders or Candle execution engines
    }
}
