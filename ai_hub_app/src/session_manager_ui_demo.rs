use ai_hub_core::session_manager::SessionManager;

pub struct DemoAppState {
    pub session_manager: SessionManager,
    pub new_session_title: String,
    pub new_session_model: String,
}

impl DemoAppState {
    pub fn new() -> Self {
        let storage_path = std::env::current_dir()
            .unwrap_or_default()
            .join("sessions");

        Self {
            session_manager: SessionManager::new(storage_path),
            new_session_title: String::new(),
            new_session_model: String::from("llama3"),
        }
    }

    /// Call this inside your main eframe::App update loop to draw the session windows
    pub fn render_session_interface(&mut self, ctx: &egui::Context) {
        // LEFT SIDEBAR PANEL FOR CONVERSATION HISTORY
        egui::SidePanel::left("session_sidebar")
            .resizable(true)
            .default_width(260.0)
            .show(ctx, |ui| {
                ui.heading("💬 Chat Sessions");
                ui.separator();

                // Interactive drawer to spawn sessions dynamically
                ui.collapsing("➕ Create New Session", |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Title:");
                        ui.text_edit_singleline(&mut self.new_session_title);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Model:");
                        ui.text_edit_singleline(&mut self.new_session_model);
                    });

                    if ui.button("✨ Initialize Session").clicked() {
                        if !self.new_session_title.is_empty() {
                            self.session_manager.create_session(
                                &self.new_session_title,
                                &self.new_session_model
                            );
                            self.new_session_title.clear();
                        }
                    }
                });

                ui.separator();
                ui.label("Saved History:");

                // Dynamic scroll window listing older logs
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let mut sessions_list: Vec<_> = self.session_manager.sessions.values().collect();
                    sessions_list.sort_by(|a, b| b.last_accessed.cmp(&a.last_accessed));

                    for session in sessions_list {
                        let is_active = Some(session.id.clone()) == self.session_manager.active_session_id;
                        if ui.selectable_label(is_active, format!("📂 {}", session.title)).clicked() {
                            self.session_manager.active_session_id = Some(session.id.clone());
                        }
                        ui.small(format!("Model: {}", session.model_name));
                    }
                });
            });

        // CENTRAL CHAT HISTORY FRAME
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(ref active_id) = self.session_manager.active_session_id {
                if let Some(session) = self.session_manager.sessions.get(active_id) {
                    ui.heading(format!("Active Thread: {}", session.title));
                    ui.separator();

                    egui::ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                        for message in &session.messages {
                            ui.horizontal(|ui| {
                                if message.role == "user" {
                                    ui.colored_label(egui::Color32::LIGHT_BLUE, "👤 You: ");
                                } else {
                                    ui.colored_label(egui::Color32::LIGHT_GREEN, "🤖 AI: ");
                                }
                                ui.label(&message.content);
                            });
                            ui.add_space(4.0);
                        }
                    });
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("👈 Select or create a chat session from the sidebar to begin.");
                });
            }
        });
    }
}
