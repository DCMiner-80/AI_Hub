use eframe::egui;
use std::sync::mpsc::Receiver;
use crate::downloader_ui::DownloadMsg;
use std::path::PathBuf;
use crate::download_scheduler::DownloadScheduler;
use std::sync::Arc;

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
            scheduler: Some(scheduler) 
        }
    }

    pub fn poll(&mut self) {
        if let Some(rx) = &self.rx {
            while let Ok(msg) = rx.try_recv() {
                match msg {
                    DownloadMsg::Progress { id, downloaded, total } => {
                        self.downloading_id = Some(id);
                        self.progress = Some((downloaded, total));
                    }
                    DownloadMsg::Done { id: _, ok } => {
                        self.progress = None;
                        self.downloading_id = None;
                        if let Err(e) = ok { 
                            eprintln!("Download failed: {}", e); 
                        }
                    }
                }
            }
        }
    }

    pub fn ui(&mut self, ctx: &egui::Context) {
        self.poll();
        egui::Window::new("Model Downloads").show(ctx, |ui| {
            ui.label("Available models:");
            if let Ok(manifest) = ai_hub_core::downloader::ModelsManifest::from_file(&self.manifest_path) {
                for m in manifest.models {
                    ui.horizontal(|ui| {
                        ui.label(format!("{} ({})", m.name, m.id));
                        if ui.button("Download now").clicked() {
                            if let Some(s) = &self.scheduler { 
                                s.enqueue(m.id.clone()); 
                            }
                        }
                    });
                }
            } else {
                ui.label("Manifest not found or invalid.");
            }
            ui.separator();
            if let Some((downloaded, total)) = self.progress {
                let pct = if total > 0 { downloaded as f32 / total as f32 } else { 0.0 };
                ui.add(egui::ProgressBar::new(pct).show_percentage());
            } else {
                ui.label("No active downloads");
            }
        });
    }
}
