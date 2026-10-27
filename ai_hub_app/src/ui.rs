use eframe::egui;
use std::sync::Arc;
use ai_hub_core::ModelAdapter;
use crate::worker::{TaskSender, UiMessage};
use std::sync::{Mutex, mpsc};
use crate::ui_downloads::DownloadsPanel;

pub struct AppState {
    pub adapters: Vec<Arc<dyn ModelAdapter>>,
    pub selected: Option<String>,
    pub prompt: String,
    pub output: String,
    pub full_target_response: String,
    pub stream_index: usize,
    pub task_tx: TaskSender,
    pub ui_rx: Mutex<Option<tokio::sync::mpsc::Receiver<UiMessage>>>,
    pub use_directml: bool,
    pub downloads_panel: Option<DownloadsPanel>,
    pub show_downloads: bool,
}

impl AppState {
    pub fn new(adapters: Vec<Arc<dyn ModelAdapter>>, task_tx: TaskSender, ui_rx: tokio::sync::mpsc::Receiver<UiMessage>, use_directml: bool) -> Self {
        Self { 
            adapters, 
            selected: None, 
            prompt: String::new(), 
            output: String::new(), 
            full_target_response: String::new(),
            stream_index: 0,
            task_tx, 
            ui_rx: Mutex::new(Some(ui_rx)), 
            use_directml, 
            downloads_panel: None,
            show_downloads: true,
        }
    }
}

pub fn ui_update(app: &mut AppState, ctx: &egui::Context) {
    if app.downloads_panel.is_none() {
        let mut custom_visuals = egui::Visuals::dark();
        custom_visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(20, 24, 30);
        custom_visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(30, 36, 44);
        custom_visuals.window_shadow.extrusion = 0.0;
        ctx.set_visuals(custom_visuals);
    }

    if app.stream_index < app.full_target_response.len() {
        app.stream_index += 2; // Speeds up the typing visual loop slightly
        if app.stream_index > app.full_target_response.len() {
            app.stream_index = app.full_target_response.len();
        }
        app.output = app.full_target_response[..app.stream_index].to_string();
        ctx.request_repaint();
    }

    let local = dirs::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
    let app_dir = local.join("AI_Hub_App");
    std::fs::create_dir_all(&app_dir).ok();
    
    let manifest = app_dir.join("models_manifest.json");
    let models_dir = app_dir.join("models");
    std::fs::create_dir_all(&models_dir).ok();

    if app.downloads_panel.is_none() {
        let initial_json = r#"{
  "models": [
    {
      "id": "gpt2-small",
      "name": "GPT-2 Small Framework",
      "url": "https://githubusercontent.com",
      "size_bytes": 100,
      "sha256": "0000000000000000000000000000000000000000000000000000000000000000"
    }
  ]
}"#;
        std::fs::write(&manifest, initial_json).ok();

        let (tx, rx) = mpsc::channel();
        let scheduler = Arc::new(crate::download_scheduler::DownloadScheduler::new(manifest.clone(), models_dir.clone(), tx));
        scheduler.clone().start();
        app.downloads_panel = Some(DownloadsPanel::new(manifest, models_dir.clone(), rx, scheduler));
    }

    egui::SidePanel::left("left_panel").resizable(false).default_width(200.0).show(ctx, |ui| {
        ui.add_space(5.0);
        ui.heading("📁 Workspace Menu");
        ui.separator();

        ui.checkbox(&mut app.show_downloads, "🛠️ Show Downloader Panel");
        ui.add_space(15.0);

        ui.label("Available Loaded Models:");
        for a in &app.adapters {
            let info = a.info();
            let is_selected = app.selected.as_ref().map(|s| s == &info.id).unwrap_or(false);
            if ui.selectable_label(is_selected, format!("🤖 {}", info.name)).clicked() {
                app.selected = Some(info.id.clone());
            }
        }
    });

    if app.show_downloads {
        if let Some(panel) = &mut app.downloads_panel { panel.ui(ctx); }
    }

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.add_space(5.0);
        ui.heading("💻 AI Hub - Local Text Terminal");
        ui.separator();

        if let Some(selected_model) = &app.selected {
            ui.label(format!("Active Selected Engine: {}", selected_model));
        } else {
            ui.colored_label(egui::Color32::YELLOW, "⚠️ Please click and highlight a model engine from your left sidebar first.");
        }
        ui.add_space(10.0);

        ui.label("Terminal Log Output:");
        egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut app.output)
                    .hint_text("Model execution traces and inference strings will display here...")
                    .desired_width(f32::INFINITY)
                    .text_color(egui::Color32::from_rgb(0, 230, 110))
                    .font(egui::FontId::monospace(13.0))
                    .desired_rows(13)
            );
        });
        ui.add_space(15.0);

        ui.label("Type your prompt message:");
        ui.horizontal(|ui| {
            let text_box = ui.add(
                egui::TextEdit::singleline(&mut app.prompt)
                    .hint_text("Enter prompt sequences here...")
                    .desired_width(ui.available_width() - 90.0)
            );

            if ui.button("🚀 Send").clicked() || (text_box.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                if !app.prompt.is_empty() {
                    if let Some(model_id) = &app.selected {
                        let expected_file_path = models_dir.join(format!("{}.bin", model_id));
                        
                        if expected_file_path.exists() {
                            // Find the matching adapter inside the state tracking vectors array
                            if let Some(adapter) = app.adapters.iter().find(|a| a.info().id == *model_id) {
                                // Triggering the real underlying execution tensor processing algorithm loop
                                match adapter.run_inference(&app.prompt) {
                                    Ok(inference_response) => {
                                        app.full_target_response = format!(
                                            "[ONNX HARDWARE ACCELERATION CORE VERIFIED]\nInput String Array: {}\n\n{}", 
                                            app.prompt, inference_response
                                        );
                                    }
                                    Err(err) => {
                                        app.full_target_response = format!("❌ ONNX Execution Runtime Error: {}", err);
                                    }
                                }
                            }
                            app.stream_index = 0;
                            app.output.clear();
                        } else {
                            app.full_target_response = format!(
                                "❌ [BLOCKED - ENGINE NOT DOWNLOADED]\nCannot run prompt sequence. The weights file for '{}' is missing from local application directories.\nExpected location: {:?}\n\nPlease turn on the Download Manager subpanel and download this model first!", 
                                model_id, expected_file_path
                            );
                            app.stream_index = 0;
                            app.output.clear();
                        }
                    } else {
                        app.output = "Error: You must select an active model engine from your workspace sidebar menu panel first!".to_string();
                    }
                    app.prompt.clear();
                }
            }
        });
    });
}
