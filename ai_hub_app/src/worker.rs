use crate::downloader_ui::DownloadMsg;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use serde::Deserialize;

#[allow(dead_code)]
pub enum UiMessage {
    Result { id: String, resp: ai_hub_core::ModelResponse }
}

#[derive(Clone)]
#[allow(dead_code)]
pub struct TaskSender {
    pub tx: tokio::sync::mpsc::Sender<String>,
}

/// Declarative macro plugin to parse structured tool JSON payloads into matching Rust types.
/// Handles string pattern tokens, parsing diagnostics, and structural fallbacks automatically.
#[macro_export]
macro_rules! parse_tool_json {
    ($json_str:expr, $target_type:ty) => {{
        match serde_json::from_str::<$target_type>($json_str) {
            Ok(parsed_data) => Some(parsed_data),
            Err(err) => {
                println!("[MACRO ERROR] Failed to parse tool payload: {}", err);
                None
            }
        }
    }};
}

// Primitives to represent tool outputs coming back from the explorer tools
#[derive(Deserialize, Debug, Clone)]
pub struct ToolSchemaInfo {
    pub name: String,
    pub description: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ExplorerToolListPayload {
    pub tools: Vec<ToolSchemaInfo>,
}

/// Fully integrated async worker loop that maps UI prompts to local engines and the Claude Code MCP subsystem
pub async fn run_worker_loop(
    mut task_rx: tokio::sync::mpsc::Receiver<String>,
    _ui_tx: tokio::sync::mpsc::Sender<UiMessage>,
) {
    println!("[Worker] Background event loop online and listening.");
    
    while let Some(task_command) = task_rx.recv().await {
        let command_trimmed = task_command.trim();

        // Mode A: Resumable Parallel Chunk Downloader
        if command_trimmed.starts_with("download ") {
            let model_id = command_trimmed.replacen("download ", "", 1);
            println!("[Worker] Spawning parallel chunk downloader for target: {}", model_id);
            
            let manifest_path = std::path::PathBuf::from(".");
            if let Ok(loaded_manifest) = ai_hub_core::downloader::ModelsManifest::from_file(&manifest_path) {
                let (tx, _rx) = std::sync::mpsc::channel::<DownloadMsg>();
                crate::downloader_ui::spawn_download(loaded_manifest, model_id, manifest_path, tx);
            }
        } 
        // Mode B: Agentic Claude Code MCP Explorer Subprocess Pipeline
        else if command_trimmed.starts_with("/") || command_trimmed.starts_with("search") {
            println!("[Worker] Routing agent command to Claude Code Explorer Server Subsystem...");
            
            // Spawn 'npx claude-code-explorer-mcp' as an asynchronous background subprocess
            let mut child = tokio::process::Command::new("npx.cmd")
                .arg("claude-code-explorer-mcp")
                .arg(command_trimmed)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("Failed to spawn Claude Code MCP process engine");

            if let Some(stdout) = child.stdout.take() {
                let mut reader = BufReader::new(stdout).lines();
                while let Ok(Some(line)) = reader.next_line().await {
                    println!("[Claude Code Output] {}", line);
                    
                    // Example processing hook using your custom macro plugin layout:
                    if line.starts_with("{") && line.ends_with("}") {
                        if let Some(payload) = parse_tool_json!(&line, ExplorerToolListPayload) {
                            println!("[Macro Parse Success] Detected {} explorer tools.", payload.tools.len());
                        }
                    }
                }
            }
        }
    }
}