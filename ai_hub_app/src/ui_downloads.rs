use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use sysinfo::{System, SystemExt};
use crate::downloader_ui::DownloadMsg;
use crate::download_scheduler::DownloadScheduler;
use crate::ui::{DownloadProgress, render_download_progress_bar};

pub struct DownloadsPanel {
    pub manifest_path: PathBuf,
    pub models_dir: PathBuf,
    pub rx: Option<Receiver<DownloadMsg>>,
    pub progress: Option<(u64, u64)>,
    pub downloading_id: Option<String>,
    pub scheduler: Option<Arc<DownloadScheduler>>,
    sys_monitor: System,
}

impl DownloadsPanel {
    pub fn new(manifest_path: PathBuf, models_dir: PathBuf, rx: Receiver<DownloadMsg>, scheduler: Arc<DownloadScheduler>) -> Self {
        let mut sys_monitor = System::new_all();
        sys_monitor.refresh_memory();
        Self {
            manifest_path,
            models_dir,
            rx: Some(rx),
            progress: None,
            downloading_id: None,
            scheduler: Some(scheduler),
            sys_monitor,
        }
    }

    pub fn poll(&mut self) {
        self.sys_monitor.refresh_memory();
        if let Some(ref rx) = self.rx {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    DownloadMsg::Progress { id, downloaded, total } => {
                        self.downloading_id = Some(id);
                        self.progress = Some((downloaded, total));
                    }
                    DownloadMsg::Done { id: _, ok: _ } => {
                        self.progress = None;
                        self.downloading_id = None;
                    }
                }
            }
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, app_state: &mut crate::ui::AppState) {
        ui.vertical(|ui| {
            ui.heading("Configuration & Download Settings");
            ui.separator();

            // Custom models directory configuration picker widget 
            ui.horizontal(|ui| {
                ui.label("Models Path:");
                ui.label(egui::RichText::new(self.models_dir.to_string_lossy()).color(egui::Color32::from_rgb(0, 255, 0)));
                if ui.button("📁 Change Location").clicked() {
                    if let Some(path) = rfd::FileDialog::new().pick_folder() {
                        self.models_dir = path;
                    }
                }
            });

            ui.add_space(10.0);

            // Hardware Telemetry Metrics Widget
            ui.group(|ui| {
                ui.heading("⚙ System Resource Allocations");
                let used_ram = self.sys_monitor.used_memory() as f64 / 1_048_576.0 / 1024.0;
                let total_ram = self.sys_monitor.total_memory() as f64 / 1_048_576.0 / 1024.0;
                
                ui.label(format!("System Memory Allocation: {:.2} GB / {:.2} GB used", used_ram, total_ram));
                
                // Hardware execution tracking feedback block
                if app_state.use_directml {
                    ui.colored_label(egui::Color32::from_rgb(0, 255, 0), "✔ GPU Core Engine Context Active (CUDA/DirectML VRAM Layer Enabled)");
                } else {
                    ui.label("Fallback processing thread: CPU Core execution channel");
                }
            });

            ui.add_space(15.0);
            ui.heading("Available Architecture Profiles");
            ui.small("Select a profile to verify local files or trigger a template-formatted prompt pre-fetch sync:");
            ui.add_space(4.0);

            if let Ok(manifest) = ai_hub_core::downloader::ModelsManifest::from_file(&self.manifest_path) {
                for model in manifest.models {
                    ui.horizontal(|ui| {
                        ui.label(format!("• {}", model.id));
                        if ui.button("⚡ Sync Profile & Format Template").clicked() {
                            // Test prompt formatting across the target model architecture profile
                            let sample_system_prompt = "You are a secure offline AI Hub interface agent.";
                            let sample_user_query = "Evaluate architecture tensors.";
                            
                            let formatted_prompt = ai_hub_core::ArchitectureTemplate::format_prompt(
                                &model.id, 
                                sample_system_prompt, 
                                sample_user_query
                            );

                            if let Some(cached_path) = ai_hub_core::LocalCandleEngine::verify_local_binary_exists(&self.models_dir, &model.name) {
                                app_state.output.clear();
                                app_state.stream_index = 0;
                                app_state.full_target_response = format!(
                                    "\n>>> [GATEKEEPER VERIFICATION] SUCCESS\n>>> {} verified locally.\n>>> Path: {}\n>>> Structured Prompt Template Applied:\n--------------------\n{}\n--------------------\n>>> Loading engine layers offline...\n",
                                    model.id, cached_path.to_string_lossy(), formatted_prompt
                                );
                            } else {
                                if let Some(ref s) = self.scheduler {
                                    app_state.output.clear();
                                    app_state.stream_index = 0;
                                    app_state.full_target_response = format!(
                                        "\n>>> [GATEKEEPER] Local cache miss for {}.\n>>> Target Template Architecture Structure:\n--------------------\n{}\n--------------------\n>>> Pulling weights from mirror endpoint...\n",
                                        model.id, formatted_prompt
                                    );
                                    s.enqueue(model.name.clone());
                                }
                            }
                        }
                    });
                }
            }
        });
    }
}
