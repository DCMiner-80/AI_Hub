use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use sysinfo::System;
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
            ui.heading("Configuration & Deployment Hub");
            ui.separator();

            // 1. Feature 1: Hermes Messaging Gateway Toggle Controls Widget
            ui.group(|ui| {
                ui.heading("🌐 Hermes Cross-Platform Messaging Gateway");
                ui.small("Deploy your agent loop into production live chat rooms instantly:");
                ui.add_space(4.0);

                let old_status = app_state.gateway_active;
                if ui.checkbox(&mut app_state.gateway_active, "Toggle Live Bot Gateways (Telegram / Discord / Slack)").changed() {
                    if app_state.gateway_active != old_status {
                        app_state.gateway_process = ai_hub_core::HermesProcessController::toggle_gateway(app_state.gateway_active);
                        
                        // Type-stream feedback state out onto your console panel view
                        app_state.output.clear();
                        app_state.stream_index = 0;
                        if app_state.gateway_active {
                            app_state.full_target_response = "\n>>> [GATEWAY DEPLOYMENT] SUCCESS\n>>> Background orchestration child thread spawned via 'hermes gateway start'\n>>> Multi-platform messaging routers successfully mapped.\n".to_string();
                        } else {
                            app_state.full_target_response = "\n>>> [GATEWAY STATUS] OFFLINE\n>>> Process handles terminated safely. Remote chat links dropped.\n".to_string();
                        }
                    }
                }

                if app_state.gateway_active {
                    ui.colored_label(egui::Color32::from_rgb(0, 255, 0), " ● Live Routing Protocol Active (Broadcasting over standard webhook listeners)");
                } else {
                    ui.weak(" ○ Gateway inactive. Bots resting offline.");
                }
            });

            ui.add_space(10.0);

            // 2. Feature 2: Interactive MCP Plugin Server Config Tool Explorer Widget
            ui.group(|ui| {
                ui.heading("🔌 Interactive Model Context Protocol (MCP) Server Tool Registry");
                ui.small("View schema endpoints exported by your active external ecosystem plug-ins:");
                ui.add_space(6.0);

                let tools = ai_hub_core::HermesProcessController::get_active_mcp_tools();
                egui::ScrollArea::vertical().max_height(140.0).show(ui, |ui| {
                    for tool in tools {
                        ui.horizontal(|ui| {
                            ui.colored_label(egui::Color32::from_rgb(0, 200, 255), format!("[{}]", tool.server_name));
                            ui.strong(format!(" {} ", tool.tool_name));
                            ui.weak(format!("- {}", tool.description));
                        });
                        ui.separator();
                    }
                });
            });

            ui.add_space(10.0);

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
            }

            ui.add_space(10.0);
            ui.heading("Available Architecture Profiles");
            if let Ok(manifest) = ai_hub_core::downloader::ModelsManifest::from_file(&self.manifest_path) {
                for model in manifest.models {
                    ui.horizontal(|ui| {
                        ui.label(format!("• {}", model.id));
                        if ui.button("⚡ Sync Profile & Format Template").clicked() {
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

