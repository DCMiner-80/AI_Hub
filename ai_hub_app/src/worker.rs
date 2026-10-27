pub enum UiMessage { Result { id: String, resp: ai_hub_core::ModelResponse } }
#[derive(Clone)]
pub struct TaskSender {
    pub tx: tokio::sync::mpsc::Sender<String>,
}
