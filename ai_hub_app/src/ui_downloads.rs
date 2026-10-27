use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
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
}

impl DownloadsPanel {
    pub fn new(manifest_path: PathBuf, models_dir: PathBuf, rx: Receiver<DownloadMsg>, scheduler: Arc<DownloadScheduler>) -> Self {
        Self {
            manifest_path,
            models_dir,
            rx: Some(rx),
            progress: None,
            downloading_id: None,
            scheduler: Some(scheduler),
        }
    }

    pub fn poll(&mut self) {
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

            if let Some((downloaded, total)) = self.progress {
                let id = self.downloading_id.clone().unwrap_or_else(|| "Unknown".to_string());
                let progress_struct = DownloadProgress {
                    file_name: id,
                    bytes_downloaded: downloaded,
                    total_bytes: total,
                };
                render_download_progress_bar(ui, &progress_struct);
            } else {
                ui.label("No active download streams running over background CDN channels.");
            }

            ui.add_space(10.0);
            ui.heading("Available Architecture Profiles");
            ui.small("Select a profile to verify local files or trigger a resumable download stream:");
            ui.add_space(4.0);

            // Fetch manifest list dynamically
            if let Ok(manifest) = ai_hub_core::downloader::ModelsManifest::from_file(&self.manifest_path) {
                for model in manifest.models {
                    ui.horizontal(|ui| {
                        ui.label(format!("• {}", model.id));
                        if ui.button("⚡ Sync Profile").clicked() {
                            if let Some(cached_path) = ai_hub_core::LocalCandleEngine::verify_local_binary_exists(&self.models_dir, &model.name) {
                                app_state.output.clear();
                                app_state.stream_index = 0;
                                app_state.full_target_response = format!(
                                    "\n>>> [GATEKEEPER VERIFICATION] SUCCESS\n>>> {} verified locally.\n>>> Path: {}\n>>> Loading engine layers offline...\n",
                                    model.id, cached_path.to_string_lossy()
                                );
                            } else {
                                if let Some(ref s) = self.scheduler {
                                    app_state.output.clear();
                                    app_state.stream_index = 0;
                                    app_state.full_target_response = format!(
                                        "\n>>> [GATEKEEPER] Local cache miss for {}.\n>>> Pulling weights from mirror endpoint...\n",
                                        model.id
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

