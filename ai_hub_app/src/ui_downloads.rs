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

    // Feature 1: Draws download fractions live onto the active panel overlay
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            ui.heading("Parallel Download Space Manager");
            ui.separator();

            if let Some((downloaded, total)) = self.progress {
                let id = self.downloading_id.clone().unwrap_or_else(|| "Unknown".to_string());
                let progress_struct = DownloadProgress {
                    file_name: id,
                    bytes_downloaded: downloaded,
                    total_bytes: total,
                };
                // Render the live egui custom progress bar component directly
                render_download_progress_bar(ui, &progress_struct);
            } else {
                ui.label("No active download streams running over background CDN channels.");
            }

            ui.add_space(10.0);
            if ui.button("Trigger Llama Manifest Pre-fetch Sync").clicked() {
                let model_url = "https://huggingface.co"; // Target model mirror URL
                
                // Fallback Verification Gatekeeper Check
                if let Some(cached_path) = ai_hub_core::LocalCandleEngine::verify_local_binary_exists(&self.models_dir, model_url) {
                    println!("Gatekeeper: Local model cache verified at {:?}. Skipping network download.", cached_path);
                    // Add direct trace confirmation line here or trigger immediate local engine loading mapping
                } else {
                    if let Some(ref s) = self.scheduler {
                        println!("Gatekeeper: Local binary not found on disk. Initiating parallel CDN streaming channels...");
                        s.enqueue("Llama-3.2-1B-Instruct-Q4_K_M".to_string());
                    }
                }
            }
        });
    }
}

