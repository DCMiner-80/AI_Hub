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
    let (task_tx, _task_rx) = tokio::sync::mpsc::channel(64);
    let (_ui_tx, ui_rx) = tokio::sync::mpsc::channel(64);
    
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

    eframe::run_native(
        "AI Hub - Desktop Model Downloader & Terminal",
        native_options,
        Box::new(move |_cc| {
            let wrapped_sender = crate::worker::TaskSender { tx: task_tx };
            let app_instance = AppState::new(active_models, wrapped_sender, ui_rx, enable_hardware_acceleration);
            Ok(Box::new(app_instance))
        })
    )
}
