use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
    pub timestamp: i64,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub model_name: String,
    pub messages: Vec<ChatMessage>,
    pub last_accessed: i64,
}

pub struct SessionManager {
    pub storage_dir: PathBuf,
    pub sessions: HashMap<String, Session>,
    pub active_session_id: Option<String>,
}

impl SessionManager {
    pub fn new<P: AsRef<Path>>(storage_path: P) -> Self {
        let storage_dir = storage_path.as_ref().to_path_buf();
        if !storage_dir.exists() {
            let _ = fs::create_dir_all(&storage_dir);
        }
        let mut manager = Self {
            storage_dir,
            sessions: HashMap::new(),
            active_session_id: None,
        };
        manager.load_all_sessions();
        manager
    }

    pub fn create_session(&mut self, title: &str, model_name: &str) -> String {
        let id = Uuid::new_v4().to_string();
        let session = Session {
            id: id.clone(),
            title: title.to_string(),
            model_name: model_name.to_string(),
            messages: Vec::new(),
            last_accessed: chrono::Utc::now().timestamp(),
        };
        self.sessions.insert(id.clone(), session);
        self.active_session_id = Some(id.clone());
        self.save_session(&id);
        id
    }

    pub fn add_message(&mut self, session_id: &str, role: &str, content: &str) {
        if let Some(session) = self.sessions.get_mut(session_id) {
            session.messages.push(ChatMessage {
                role: role.to_string(),
                content: content.to_string(),
                timestamp: chrono::Utc::now().timestamp(),
            });
            session.last_accessed = chrono::Utc::now().timestamp();
            self.save_session(session_id);
        }
    }

    fn save_session(&self, id: &str) {
        if let Some(session) = self.sessions.get(id) {
            if let Ok(json) = serde_json::to_string_pretty(session) {
                if let Ok(mut file) = File::create(self.storage_dir.join(format!("{}.json", id))) {
                    let _ = file.write_all(json.as_bytes());
                }
            }
        }
    }

    fn load_all_sessions(&mut self) {
        if let Ok(entries) = fs::read_dir(&self.storage_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(session) = serde_json::from_str::<Session>(&content) {
                            self.sessions.insert(session.id.clone(), session);
                        }
                    }
                }
            }
        }
    }
}
