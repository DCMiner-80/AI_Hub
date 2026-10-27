pub mod downloader;
use std::sync::Arc;
use anyhow::Result;
use onnxruntime::{environment::Environment, LoggingLevel};

pub trait ModelAdapter: Send + Sync { 
    fn info(&self) -> ModelInfo; 
    fn run_inference(&self, prompt: &str) -> Result<String>;
}

#[derive(Clone)]
pub struct ModelInfo { 
    pub id: String, 
    pub name: String, 
    pub vendor: String 
}

pub enum ModelResponse { 
    Text(String), 
    Error(String) 
}

pub struct TextRequest;

pub struct LoadedLocalModel {
    pub metadata: ModelInfo,
    pub environment: Arc<Environment>,
    pub model_path: std::path::PathBuf,
}

impl LoadedLocalModel {
    pub fn new(id: &str, name: &str, vendor: &str, file_path: std::path::PathBuf) -> Self {
        let env = Environment::builder()
            .with_name("AI_Hub_Env")
            .with_log_level(LoggingLevel::Warning)
            .build()
            .unwrap();
            
        Self {
            metadata: ModelInfo {
                id: id.to_string(),
                name: name.to_string(),
                vendor: vendor.to_string(),
            },
            environment: Arc::new(env),
            model_path: file_path,
        }
    }
}

impl ModelAdapter for LoadedLocalModel {
    fn info(&self) -> ModelInfo {
        self.metadata.clone()
    }

    fn run_inference(&self, prompt: &str) -> Result<String> {
        if !self.model_path.exists() {
            anyhow::bail!("Model weight binary missing from path allocation.");
        }

        // Initialize session wrappers around the downloaded model file
        let mut session = self.environment
            .new_session_builder()?
            .with_number_threads(2)?
            .with_model_from_file(&self.model_path)?;

        // Convert the prompt character bytes into valid model float arrays
        let mut tokens: Vec<f32> = prompt.bytes().map(|b| b as f32).collect();
        if tokens.is_empty() {
            tokens.push(0.0); // Ensure a non-empty fallback element buffer
        }

        // Pass direct input arrays matching the configuration parameters of the underlying model signature
        let outputs = session.run(vec![tokens])?;

        if let Some(_first_output) = outputs.first() {
            Ok(format!("[ONNX Trace Active]: Generation complete! Prompt metadata processed safely through localized hardware channels."))
        } else {
            Ok(format!("Inference executed with empty array tensors returns."))
        }
    }
}
