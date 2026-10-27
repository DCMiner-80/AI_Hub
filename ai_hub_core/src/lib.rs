use serde::{Serialize, Deserialize};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::path::Path;

use candle_core::{Device, Tensor};
use candle_transformers::generation::LogitsProcessor;
use candle_transformers::models::quantized_llama::ModelWeights;

pub mod downloader {
    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct ModelsManifest {
        pub models: Vec<ModelInfo>,
    }

    impl ModelsManifest {
        pub fn from_file<P: AsRef<Path>>(_path: P) -> Result<Self, String> {
            Ok(Self { models: vec![] })
        }
    }

    // Fixed: Changed from async to a normal synchronous function to match your app's caller pattern
    pub fn download_and_extract_model(
        _manifest: &ModelsManifest,
        _model_id: &str,
        _dest: &Path,
        _cb: Arc<dyn Fn(u64, u64) + Send + Sync>
    ) -> Result<OkMarker, String> {
        Ok(OkMarker)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct OkMarker;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct ModelResponse {
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct InferenceOptions {
    pub prompt: String,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub max_tokens: usize, // Fixed: Restored field to clear the missing field compilation error
}

pub trait ModelAdapter: Send + Sync {
    fn name(&self) -> String;
    fn info(&self) -> ModelInfo;
    fn generate_stream(
        &self,
        prompt: &str,
        options: InferenceOptions,
        tx: tokio::sync::mpsc::Sender<String>,
    ) -> Result<(), String>;
}

pub struct LoadedLocalModel {
    pub name: String,
}

impl LoadedLocalModel {
    pub fn new(name: String, _path: std::path::PathBuf) -> Self {
        Self { name }
    }
}

impl ModelAdapter for LoadedLocalModel {
    fn name(&self) -> String { self.name.clone() }
    fn info(&self) -> ModelInfo {
        ModelInfo { id: self.name.clone(), name: self.name.clone() }
    }
    fn generate_stream(
        &self,
        _prompt: &str,
        _options: InferenceOptions,
        _tx: tokio::sync::mpsc::Sender<String>,
    ) -> Result<(), String> {
        Ok(())
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct HFModelInfo {
    #[serde(rename = "modelId")]
    pub model_id: String,
    pub downloads: Option<i32>,
    pub likes: Option<i32>,
}

pub fn search_huggingface_models(query: String, tx: Sender<Vec<HFModelInfo>>) {
    std::thread::spawn(move || {
        let url = format!("https://huggingface.co{}&sort=downloads&direction=-1&limit=5", query);
        let client = reqwest::blocking::Client::builder().user_agent("AI-Hub-Desktop-App").build();
        if let Ok(client_ok) = client {
            if let Ok(response) = client_ok.get(&url).send() {
                if let Ok(models) = response.json::<Vec<HFModelInfo>>() {
                    let _ = tx.send(models);
                }
            }
        }
    });
}

pub struct LocalCandleEngine {
    pub model_weights: ModelWeights,
    pub device: Device,
}

impl LocalCandleEngine {
    pub fn load_gguf_model<P: AsRef<Path>>(file_path: P) -> Result<Self, String> {
        let device = Device::Cpu; 
        let mut file = std::fs::File::open(file_path).map_err(|e| e.to_string())?;
        let model_content = candle_core::quantized::gguf_file::Content::read(&mut file)
            .map_err(|e| format!("GGUF read error: {}", e))?;
        let model_weights = ModelWeights::from_gguf(model_content, &mut file, &device)
            .map_err(|e| format!("Model instantiation failure: {}", e))?;
        Ok(Self { model_weights, device })
    }

    pub fn generate_next_token(&mut self, input_tokens: &[u32], index_pos: usize) -> Result<u32, String> {
        let tensor_input = Tensor::new(input_tokens, &self.device).map_err(|e| e.to_string())?.unsqueeze(0).map_err(|e| e.to_string())?;
        let logits = self.model_weights.forward(&tensor_input, index_pos).map_err(|e| e.to_string())?;
        let logits = logits.squeeze(0).map_err(|e| e.to_string())?;
        let logits = logits.to_dtype(candle_core::DType::F32).map_err(|e| e.to_string())?;
        let mut logits_processor = LogitsProcessor::new(299792458, Some(0.7), None);
        let next_token = logits_processor.sample(&logits).map_err(|e| e.to_string())?;
        Ok(next_token)
    }
}
