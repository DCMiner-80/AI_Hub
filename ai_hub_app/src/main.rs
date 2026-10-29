use std::sync::Arc;
use ai_hub_core::{LoadedLocalModel, HermesAgentAdapter};

mod ui;
mod worker;
mod downloader_ui;
mod ui_downloads;
mod download_scheduler;

use crate::ui::AppState;

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        crate::ui::ui_update(self, ctx);
    }
}

fn main() -> eframe::Result<()> {
    // 1. Initialize a multi-threaded Tokio runtime for background execution
    let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
    
    // 2. Properly instantiate the cross-thread communication channels
    let (task_tx, task_rx) = tokio::sync::mpsc::channel(64);
    let (ui_tx, ui_rx) = tokio::sync::mpsc::channel(64);
    
    // 3. Spawn the background processing worker inside the async runtime loop
    rt.spawn(async move {
        crate::worker::run_worker_loop(task_rx, ui_tx).await;
    });

    let enable_hardware_acceleration = true;
    let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_else(|_| ".".to_string());
    let hermes_base_dir = std::path::PathBuf::from(local_app_data).join("hermes");

    // Connect both standard Candle models and the autonomous Hermes Agent loop side-by-side
    let active_models: Vec<Arc<dyn ai_hub_core::ModelAdapter>> = vec![
        Arc::new(LoadedLocalModel::new(
            "Llama-3.2-1B-Instruct-Local-Engine".to_string(),
            std::path::PathBuf::from("."),
            enable_hardware_acceleration
        )),
        Arc::new(HermesAgentAdapter::new(hermes_base_dir))
    ];

    let mut native_options = eframe::NativeOptions::default();
    native_options.viewport = egui::ViewportBuilder::default()
        .with_inner_size(egui::vec2(1024.0, 768.0));

    // 4. Start the blocking native UI loop on the main OS thread
    eframe::run_native(
        "AI Hub - Desktop Model Downloader & Terminal",
        native_options,
        Box::new(move |_cc| {
            let wrapped_sender = crate::worker::TaskSender { tx: task_tx };
            let mut app_instance = AppState::new(active_models, wrapped_sender, ui_rx, enable_hardware_acceleration);
            
            // 5. Explicitly instantiate the downloader pipeline channel context
            let (dl_tx, dl_rx) = std::sync::mpsc::channel();
            let scheduler = std::sync::Arc::new(crate::download_scheduler::DownloadScheduler::new(
                std::path::PathBuf::from("."), 
                std::path::PathBuf::from("."), 
                dl_tx
            ));
            app_instance.downloads_panel = Some(crate::ui_downloads::DownloadsPanel::new(
                std::path::PathBuf::from("."), 
                std::path::PathBuf::from("."), 
                dl_rx, 
                scheduler
            ));

            Ok(Box::new(app_instance))
        })
    )
}