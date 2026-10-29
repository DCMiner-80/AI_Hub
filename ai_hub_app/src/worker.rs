use crate::downloader_ui::DownloadMsg;

#[allow(dead_code)]
pub enum UiMessage {
    Result { id: String, resp: ai_hub_core::ModelResponse }
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct TaskSender {
    pub tx: tokio::sync::mpsc::Sender<String>,
}

/// Fully operational async worker loop handling background triggers
pub async fn run_worker_loop(
    mut task_rx: tokio::sync::mpsc::Receiver<String>,
    _ui_tx: tokio::sync::mpsc::Sender<UiMessage>,
) {
    println!("[Worker] Background event loop online.");
    
    while let Some(task_command) = task_rx.recv().await {
        println!("[Worker] Received active prompt command: {}", task_command);
        let command_trimmed = task_command.trim();

        if command_trimmed.starts_with("download ") {
            let model_id = command_trimmed.replacen("download ", "", 1);
            println!("[Worker] Triggering parallel chunk downloader for target: {}", model_id);
            
            // 1. Correctly initialize the manifest via its public direct module context route
            let manifest_path = std::path::PathBuf::from(".");
            if let Ok(loaded_manifest) = ai_hub_core::downloader::ModelsManifest::from_file(&manifest_path) {
                
                // 2. Invoke your background downloader engine with all 4 required arguments
                let (tx, _rx) = std::sync::mpsc::channel::<DownloadMsg>();
                crate::downloader_ui::spawn_download(
                    loaded_manifest,
                    model_id,
                    manifest_path,
                    tx
                );
            } else {
                println!("[Worker] Error: Failed to initialize ModelsManifest context.");
            }
        }
    }
}