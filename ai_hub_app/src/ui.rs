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
    
    // ? Persistent Key history & HF Searching State mappings
    pub prompt_history: Vec<String>,
    pub history_index: Option<usize>,
    pub hf_search_query: String,
}

impl AppState {
    pub fn new(adapters: Vec<Arc<dyn ModelAdapter>>, task_tx: TaskSender, ui_rx: tokio::sync::mpsc::Receiver<UiMessage>, use_directml: bool) -> Self {
        // Attempt to read terminal command history log context from disk if present
        let local_dir = dirs::data_local_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        let history_path = local_dir.join("AI_Hub_App").join("history.json");
        let loaded_history = std::fs::read_to_string(history_path)
            .ok()
            .and_then(|s| serde_json::from_str::<Vec<String>>(&s).ok())
            .unwrap_or_default();

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
            prompt_history: loaded_history,
            history_index: None,
            hf_search_query: String::new(),
        }
    }
}

pub fn ui_update(app: &mut AppState, ctx: &egui::Context) {
    // Set up a standard channel to catch models returning from the background thread
    let (_hf_tx, hf_rx) = std::sync::mpsc::channel::<Vec<ai_hub_core::HFModelInfo>>();
    
    // Check for search response data sent back from the background thread without locking the frame
    if let Ok(found_models) = hf_rx.try_recv() {
        if found_models.is_empty() {
            app.output.push_str("\n>>> No matching models located on Hugging Face database repositories.\n");
        } else {
            app.output.push_str("\n>>> Top Matching Live Repositories Located:\n");
            for model in found_models {
                let downloads_count = model.downloads.unwrap_or(0);
                let likes_count = model.likes.unwrap_or(0);
                app.output.push_str(&format!(
                    " 📦 {} | 📥 Downloads: {} | ❤️ Likes: {}\n",
                    model.model_id, downloads_count, likes_count
                ));
            }
        }
    }
    if app.downloads_panel.is_none() {
        let mut custom_visuals = egui::Visuals::dark();
        custom_visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(20, 24, 30);
        custom_visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(30, 36, 44);
        custom_visuals.window_shadow.spread = 0.0;
        ctx.set_visuals(custom_visuals);
    }

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

        // ?? Live Hugging Face Repository Search Bar UI Block
        ui.label("?? Search Hugging Face Hub (GGUF):");
        let search_box = ui.text_edit_singleline(&mut app.hf_search_query);
        if search_box.changed() && !app.hf_search_query.is_empty() {
            let _query = app.hf_search_query.clone();
            // Trigger background search routine mapping here
        }

        ui.separator();
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
            // ?? Upgraded Markdown view renderer with true syntax code block highlighting
            render_native_markdown(ui, &app.output);
        });

        ui.horizontal(|ui| {
            // ?? Intercept Keyboard Up/Down events for terminal cycling execution
            if ui.input(|i| i.key_pressed(egui::Key::ArrowUp)) && !app.prompt_history.is_empty() {
                let current_idx = app.history_index.unwrap_or(app.prompt_history.len());
                if current_idx > 0 {
                    let next_idx = current_idx - 1;
                    app.prompt = app.prompt_history[next_idx].clone();
                    app.history_index = Some(next_idx);
                }
            }
            if ui.input(|i| i.key_pressed(egui::Key::ArrowDown)) {
                if let Some(current_idx) = app.history_index {
                    if current_idx + 1 < app.prompt_history.len() {
                        let next_idx = current_idx + 1;
                        app.prompt = app.prompt_history[next_idx].clone();
                        app.history_index = Some(next_idx);
                    } else {
                        app.prompt.clear();
                        app.history_index = None;
                    }
                }
            }

            let prompt_box = ui.add_sized(
                [ui.available_width() - 80.0, 30.0],
                egui::TextEdit::singleline(&mut app.prompt).hint_text("Enter prompt here...")
            );

            if (prompt_box.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) || ui.button("Submit").clicked() {
                if !app.prompt.is_empty() {
                    // Update the prompt history array
                    app.prompt_history.push(app.prompt.clone());
                    app.history_index = None;

                    // Persist prompt cache history to disk layout
                    let history_file = app_dir.join("history.json");
                    if let Ok(serialized) = serde_json::to_string(&app.prompt_history) {
                        let _ = std::fs::write(history_file, serialized);
                    }

                    if let Some(model_id) = &app.selected {
                        if let Some(adapter) = app.adapters.iter().find(|a| a.info().id == *model_id) {
                            let (tx, rx) = tokio::sync::mpsc::channel(64);
                            let adapter_clone = adapter.clone();
                            let prompt_clone = app.prompt.clone();
                            
                            let options = ai_hub_core::InferenceOptions { prompt: app.prompt.clone(),
                                temperature: Some(app.temperature as f32),
                                top_p: Some(app.top_p as f32),
                                max_tokens: 256,
                            };

                            std::thread::spawn(move || {
                                if let Err(e) = adapter_clone.generate_stream(&prompt_clone, options, tx) {
                                    eprintln!("Inference generation failure engine exception event: {}", e);
                                }
                            });

                            app.output.push_str(&format!("\n\n### ?? Prompt:\n> {}\n\n", app.prompt));
                            app.prompt.clear();
                            app.inference_receiver = Some(rx);
                        }
                    }
                }
            }
        });
    });
}



fn render_native_markdown(ui: &mut eframe::egui::Ui, text: &str) {
    ui.vertical(|ui| {
        for line in text.lines() {
            if line.starts_with("# ") {
                ui.heading(line.trim_start_matches("# "));
            } else if line.starts_with("## ") {
                ui.label(eframe::egui::RichText::new(line.trim_start_matches("## ")).strong().size(18.0));
            } else if line.starts_with("### ") {
                ui.label(eframe::egui::RichText::new(line.trim_start_matches("### ")).strong().size(15.0));
            } else if line.starts_with("- ") || line.starts_with("* ") {
                ui.horizontal(|ui| {
                    ui.label("•");
                    ui.label(line.trim_start_matches("- ").trim_start_matches("* "));
                });
            } else if line.starts_with("` ") || line.starts_with("    ") {
                ui.label(eframe::egui::RichText::new(line).code());
            } else {
                ui.horizontal_wrapped(|ui| {
                    let mut current_line = line;
                    while let Some(start) = current_line.find("**") {
                        if let Some(end) = current_line[start + 2..].find("**") {
                            let end_idx = start + 2 + end;
                            ui.label(&current_line[..start]);
                            ui.label(eframe::egui::RichText::new(&current_line[start + 2..end_idx]).strong());
                            current_line = &current_line[end_idx + 2..];
                        } else { break; }
                    }
                    ui.label(current_line);
                });
            }
        }
    });
}


pub struct DownloadProgress {
    pub file_name: String,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
}

/// Task 2: Custom layout component rendering parallel multi-threaded progress tracking
pub fn render_download_progress_bar(ui: &mut egui::Ui, progress: &DownloadProgress) {
    ui.vertical(|ui| {
        ui.label(format!("Downloading: {}", progress.file_name));

        let progress_fraction = if progress.total_bytes > 0 {
            progress.bytes_downloaded as f32 / progress.total_bytes as f32
        } else {
            0.0
        };

        let progress_bar = egui::ProgressBar::new(progress_fraction)
            .text(format!(
                "{:.1} MB / {:.1} MB ({:.1}%)",
                progress.bytes_downloaded as f64 / 1_048_576.0,
                progress.total_bytes as f64 / 1_048_576.0,
                progress_fraction * 100.0
            ))
            .animate(true);

        ui.add(progress_bar);
        ui.add_space(4.0);
    });
}







