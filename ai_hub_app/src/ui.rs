use eframe::egui;
use std::sync::Arc;
use ai_hub_core::ModelAdapter;
use crate::worker::{TaskSender, UiMessage};
use std::sync::{Mutex, mpsc};
use crate::ui_downloads::DownloadsPanel;

#[allow(dead_code)]
pub struct AppState {
    pub system_prompt: String,
    pub temperature: f64,
    pub top_p: f64,
    pub inference_receiver: Option<tokio::sync::mpsc::Receiver<String>>,
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
            system_prompt: "You are a helpful AI assistant.".to_string(),
            temperature: 0.7,
            top_p: 0.9,
            inference_receiver: None,
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

    // ? Real-time non-blocking token harvester loop running on every single frame tick
    if let Some(ref mut rx) = app.inference_receiver {
        let mut got_new_tokens = false;
        while let Ok(token_fragment) = rx.try_recv() {
            app.output.push_str(&token_fragment);
            got_new_tokens = true;
        }
        if got_new_tokens {
            ctx.request_repaint();
        }
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
        ui.heading("?? Workspace Menu");
        ui.separator();

        ui.checkbox(&mut app.show_downloads, "? Show Downloader Panel");
        ui.add_space(15.0);

        ui.label("Available Loaded Models:");
        for a in &app.adapters {
            let info = a.info();
            let is_selected = app.selected.as_ref().map(|s| s == &info.id).unwrap_or(false);
            if ui.selectable_label(is_selected, format!("?? {}", info.name)).clicked() {
                app.selected = Some(info.id.clone());
            }
        }
        
        ui.add_space(15.0);
        ui.separator();
        
        // ? Generation Settings controllers neatly nested inside side panel layout
        ui.collapsing("? Generation Settings", |ui| {
            ui.label("System Prompt:");
            ui.text_edit_multiline(&mut app.system_prompt);
            
            ui.separator();
            
            ui.label(format!("Temperature: {:.2}", app.temperature));
            ui.add(egui::Slider::new(&mut app.temperature, 0.0..=1.5).show_value(false));
            
            ui.label(format!("Top-P: {:.2}", app.top_p));
            ui.add(egui::Slider::new(&mut app.top_p, 0.0..=1.0).show_value(false));
        });
    });

    if app.show_downloads {
        if let Some(panel) = &mut app.downloads_panel { panel.ui(ctx); }
    }

    egui::CentralPanel::default().show(ctx, |ui| {
        ui.heading("? Interactive Local Terminal");
        ui.separator();

        let layout_height = ui.available_height() - 40.0;
        egui::ScrollArea::vertical().max_height(layout_height).stick_to_bottom(true).show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(&mut app.output)
                    .font(egui::TextStyle::Monospace)
                    .frame(false)
                    .desired_width(f32::INFINITY)
            );
        });

        ui.horizontal(|ui| {
            let prompt_box = ui.add_sized(
                [ui.available_width() - 80.0, 30.0],
                egui::TextEdit::singleline(&mut app.prompt).hint_text("Enter prompt here...")
            );

            if (prompt_box.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) || ui.button("Submit").clicked() {
                if !app.prompt.is_empty() {
                    if let Some(model_id) = &app.selected {
                        if let Some(adapter) = app.adapters.iter().find(|a| a.info().id == *model_id) {
                            let (tx, rx) = tokio::sync::mpsc::channel(64);
                            let adapter_clone = adapter.clone();
                            let prompt_clone = app.prompt.clone();
                            
                            let options = ai_hub_core::InferenceOptions {
                                temperature: app.temperature,
                                top_p: app.top_p,
                                max_tokens: 256,
                            };

                            std::thread::spawn(move || {
                                if let Err(e) = adapter_clone.generate_stream(&prompt_clone, options, tx) {
                                    eprintln!("Inference generation failure engine exception event: {}", e);
                                }
                            });

                            app.output.push_str(&format!("\n> {}\n", app.prompt));
                            app.prompt.clear();
                            app.inference_receiver = Some(rx);
                        }
                    }
                }
            }
        });
    });
}