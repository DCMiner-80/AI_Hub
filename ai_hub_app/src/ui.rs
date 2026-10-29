use eframe::egui;
use std::sync::Arc;
use ai_hub_core::ModelAdapter;
use crate::worker::{TaskSender, UiMessage};
use std::sync::Mutex;
use crate::ui_downloads::DownloadsPanel;

#[derive(Clone)]
pub struct DownloadProgress {
    pub file_name: String,
    pub bytes_downloaded: u64,
    pub total_bytes: u64,
    pub speed_mbps: f64,
}

pub fn render_download_progress_bar(ui: &mut egui::Ui, progress: &DownloadProgress) {
    ui.vertical(|ui| {
        ui.label(format!("File: {}", progress.file_name));
        let pct = if progress.total_bytes > 0 { progress.bytes_downloaded as f32 / progress.total_bytes as f32 } else { 0.0 };
        ui.horizontal(|ui| {
            ui.add(egui::ProgressBar::new(pct).show_percentage().desired_width(ui.available_width() - 80.0));
            ui.small(format!("{:.2} MB/s", progress.speed_mbps));
        });
    });
}

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
    pub gateway_active: bool,
    pub gateway_process: Option<std::process::Child>,
    pub prompt_history: Vec<String>,
    pub history_index: Option<usize>,
    pub hf_search_query: String,
    pub hf_rx: std::sync::mpsc::Receiver<Vec<ai_hub_core::HFModelInfo>>,
    pub hf_tx: std::sync::mpsc::Sender<Vec<ai_hub_core::HFModelInfo>>,
}

impl AppState {
    pub fn new(adapters: Vec<Arc<dyn ModelAdapter>>, task_tx: TaskSender, ui_rx: tokio::sync::mpsc::Receiver<UiMessage>, use_directml: bool) -> Self {
        let (hf_tx, hf_rx) = std::sync::mpsc::channel();
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
            gateway_active: false,
            gateway_process: None,
            downloads_panel: None,
            show_downloads: true,
            prompt_history: Vec::new(),
            history_index: None,
            hf_search_query: String::new(),
            hf_tx,
            hf_rx,
        }
    }
}

pub fn ui_update(app: &mut AppState, ctx: &egui::Context) {
    // 1. TOP GLOBAL MENU BAR
    egui::TopBottomPanel::top("global_taskbar_menu").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("📁 File", |ui| {
                if ui.button("❌ Close Application").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });
            ui.checkbox(&mut app.show_downloads, "📥 Show Downloader Window");
        });
    });

    // 2. ACTIVE PROGRESS QUEUE POLLING
    if let Some(panel) = &mut app.downloads_panel {
        panel.poll();
    }

    if let Some(ref mut rx) = app.inference_receiver {
        while let Ok(token_fragment) = rx.try_recv() {
            app.output.push_str(&token_fragment);
            ctx.request_repaint();
        }
    }

    // 3. LAYOUT MANAGER PANEL SPLITTER
    egui::CentralPanel::default().show(ctx, |ui| {
        if app.show_downloads {
            ui.columns(2, |columns| {
                // Left Side: Interactive Shell Console Terminal
                columns[0].vertical(|ui| {
                    ui.heading("💻 AI Hub Core Matrix Terminal");
                    egui::ScrollArea::vertical().id_source("console_scroll").max_height(350.0).show(ui, |ui| {
                        ui.add(egui::TextEdit::multiline(&mut app.output).font(egui::FontId::monospace(12.0)).desired_width(f32::INFINITY));
                    });
                    
                    ui.separator();
                    ui.horizontal(|ui| {
                        let res = ui.add(egui::TextEdit::singleline(&mut app.prompt).desired_width(ui.available_width() - 80.0));
                        if ui.button("🚀 Run").clicked() || (res.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                            if !app.prompt.trim().is_empty() {
                                let input_prompt = app.prompt.clone();
                                app.output.push_str(&format!("\n>>> {}\n", input_prompt));
                                app.prompt.clear();
                                let _ = app.task_tx.tx.try_send(input_prompt);
                            }
                        }
                    });
                });

                // Right Side: Safe Borrow Panel Isolation Container
                columns[1].vertical(|ui| {
                    ui.heading("📥 Download Control Management");
                    
                    // Temporarily take the panel out of the state tree to resolve borrow alignment conflicts
                    if let Some(mut panel) = app.downloads_panel.take() {
                        panel.ui(ui, app);
                        // Safely return it to the active structure tree immediately after draw
                        app.downloads_panel = Some(panel);
                    } else {
                        ui.weak("Initialising parallel download worker layers...");
                    }
                });
            });
        } else {
            // Full Screen Console Execution View Map
            ui.heading("💻 AI Hub Core Matrix Terminal");
            egui::ScrollArea::vertical().id_source("full_console_scroll").max_height(450.0).show(ui, |ui| {
                ui.add(egui::TextEdit::multiline(&mut app.output).font(egui::FontId::monospace(13.0)).desired_width(f32::INFINITY));
            });
            ui.separator();
            ui.horizontal(|ui| {
                let res = ui.add(egui::TextEdit::singleline(&mut app.prompt).desired_width(ui.available_width() - 80.0));
                if ui.button("🚀 Run").clicked() || (res.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))) {
                    if !app.prompt.trim().is_empty() {
                        let input_prompt = app.prompt.clone();
                        app.output.push_str(&format!("\n>>> {}\n", input_prompt));
                        app.prompt.clear();
                        let _ = app.task_tx.tx.try_send(input_prompt);
                    }
                }
            });
        }
    });
}