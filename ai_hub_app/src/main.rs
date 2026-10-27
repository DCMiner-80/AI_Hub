#![windows_subsystem = "windows"]

mod ui;
mod worker;
mod downloader_ui;
mod download_scheduler;
mod ui_downloads;

use crate::ui::{AppState, ui_update};
use ai_hub_core::LoadedLocalModel;
use std::sync::Arc;

fn main() -> Result<(), eframe::Error> {
    let (task_tx, _task_rx) = tokio::sync::mpsc::channel(32);
    let (_ui_tx, ui_rx) = tokio::sync::mpsc::channel(32);
    
    let task_sender_wrapped = worker::TaskSender { tx: task_tx };
    
    let local = dirs::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let models_dir = local.join("AI_Hub_App").join("models");

    // Register active ONNX backends into your state controller
    let active_models: Vec<Arc<dyn ai_hub_core::ModelAdapter>> = vec![
        Arc::new(LoadedLocalModel::new(
            "gpt2-small",
            "GPT-2 Small Framework",
            "OpenAI",
            models_dir.join("gpt2-small.bin")
        ))
    ];
    
    let app_state = AppState::new(
        active_models, 
        task_sender_wrapped, 
        ui_rx, 
        false 
    );

    let options = eframe::NativeOptions {
        initial_window_size: Some(eframe::egui::vec2(800.0, 600.0)),
        ..Default::default()
    };

    eframe::run_native(
        "AI Hub - Model Downloader",
        options,
        Box::new(|_cc| {
            struct MainApp {
                state: AppState,
            }
            impl eframe::App for MainApp {
                fn update(&mut self, ctx: &eframe::egui::Context, _frame: &mut eframe::Frame) {
                    ui_update(&mut self.state, ctx);
                }
            }
            Box::new(MainApp { state: app_state })
        }),
    )
}
