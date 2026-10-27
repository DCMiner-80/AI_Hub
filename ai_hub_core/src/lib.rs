use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::mpsc::Sender;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncSeekExt, AsyncWriteExt};
use candle_core::{Device, Tensor};
use candle_transformers::generation::LogitsProcessor;
use tokenizers::Tokenizer;
use futures_util::StreamExt;

// --- EXPLICIT COMPATIBILITY WITH ORIGINAL DOWNLOADER API ---
pub mod downloader {
    use super::*;
    use std::path::Path;
    use std::sync::Arc;

    #[derive(Clone)]
    pub struct ManifestModel {
        pub id: String,
        pub name: String,
        pub repo: String,
        pub filename: String,
        pub size_gb: f32,
    }

    #[derive(Clone)]
    pub struct ModelsManifest {
        pub models: Vec<ManifestModel>,
    }

    impl ModelsManifest {
        pub fn from_file(_path: &Path) -> Result<Self, String> {
            Ok(Self {
                models: vec![
                    ManifestModel {
                        id: "qwen2.5-1.5b-instruct".to_string(),
                        name: "Qwen 2.5 1.5B Instruct (GGUF Q4_K_M)".to_string(),
                        repo: "Qwen/Qwen2.5-1.5B-Instruct-GGUF".to_string(),
                        filename: "qwen2.5-1.5b-instruct-q4_k_m.gguf".to_string(),
                        size_gb: 1.15,
                    },
                    ManifestModel {
                        id: "phi-3.5-mini-instruct".to_string(),
                        name: "Microsoft Phi-3.5 Mini Instruct (GGUF Q4_K_M)".to_string(),
                        repo: "bartowski/Phi-3.5-mini-instruct-GGUF".to_string(),
                        filename: "Phi-3.5-mini-instruct-Q4_K_M.gguf".to_string(),
                        size_gb: 2.43,
                    },
                ],
            })
        }
    }
    
    pub fn download_and_extract_model(
        manifest: &ModelsManifest,
        model_id: &str,
        dest_dir: &Path,
        callback: Arc<dyn Fn(u64, u64) + Send + Sync>
    ) -> Result<(), String> {
        let model = manifest.models.iter()
            .find(|m| m.id == model_id)
            .ok_or_else(|| "Selected Model target ID not found in system manifest listings".to_string())?;

        let url = format!("https://huggingface.co{}/resolve/main/{}", model.repo, model.filename);
        let output_path = dest_dir.join(&model.filename);

        let rt = tokio::runtime::Runtime::new().map_err(|e| e.to_string())?;
        
        rt.block_on(async {
            let client = reqwest::Client::new();
            
            // 1. Discover full byte length size from target endpoint headers
            let response = client.head(&url).send().await.map_err(|e| e.to_string())?;
            let total_size = response.headers()
                .get(reqwest::header::CONTENT_LENGTH)
                .and_then(|ct| ct.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok())
                .ok_or_else(|| "Failed to obtain content length range properties from host remote server".to_string())?;

            // 2. Provision empty destination container file map on native Windows storage path
            tokio::fs::create_dir_all(dest_dir).await.map_err(|e| e.to_string())?;
            let file = OpenOptions::new()
                .write(true)
                .create(true)
                .open(&output_path)
                .await
                .map_err(|e| e.to_string())?;
            file.set_len(total_size).await.map_err(|e| e.to_string())?;

            // 3. Orchestrate 4 distributed chunk parallel workers using Range Headers
            let chunk_count = 4u64;
            let chunk_size = total_size / chunk_count;
            let downloaded_bytes = Arc::new(AtomicU64::new(0));
            let mut tasks = vec![];

            for i in 0..chunk_count {
                let start = i * chunk_size;
                let end = if i == chunk_count - 1 { total_size - 1 } else { start + chunk_size - 1 };
                
                let client_clone = client.clone();
                let url_clone = url.clone();
                let path_clone = output_path.clone();
                let downloaded_counter = downloaded_bytes.clone();
                let cb_clone = callback.clone();

                tasks.push(tokio::spawn(async move {
                    let req = client_clone.get(&url_clone)
                        .header(reqwest::header::RANGE, format!("bytes={}-{}", start, end));
                    
                    if let Ok(res) = req.send().await {
                        let mut stream = res.bytes_stream();
                        if let Ok(mut out_file) = OpenOptions::new().write(true).open(&path_clone).await {
                            if out_file.seek(std::io::SeekFrom::Start(start)).await.is_ok() {
                                while let Some(Ok(bytes_chunk)) = stream.next().await {
                                    let chunk_data = bytes_chunk;
                                    if out_file.write_all(&chunk_data).await.is_ok() {
                                        let len = chunk_data.len() as u64;
                                        let overall = downloaded_counter.fetch_add(len, Ordering::SeqCst) + len;
                                        cb_clone(overall, total_size);
                                    } else { break; }
                                }
                            }
                        }
                    }
                }));
            }

            for task in tasks {
                let _ = task.await;
            }
            
            Ok(())
        })
    }
}

#[derive(Clone)]
pub struct ModelResponse {
    pub text: String,
}

pub struct ModelInfo {
    pub id: String,
    pub name: String,
}

pub trait ModelAdapter: Send + Sync {
    fn info(&self) -> ModelInfo;
    fn run_inference(&self, prompt: &str) -> Result<ModelResponse, String>;
    fn load(&mut self, model_path: &Path, tokenizer_path: &Path) -> Result<(), String>;
    fn generate_stream(
        &self,
        prompt: &str,
        options: InferenceOptions,
        tx: Sender<String>,
    ) -> Result<(), String>;
}

pub struct LoadedLocalModel {
    pub id: String,
    pub name: String,
    pub author: String,
    pub path: PathBuf,
}

impl LoadedLocalModel {
    pub fn new(id: &str, name: &str, author: &str, path: PathBuf) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            author: author.to_string(),
            path,
        }
    }
}

impl ModelAdapter for LoadedLocalModel {
    fn info(&self) -> ModelInfo {
        ModelInfo {
            id: self.id.clone(),
            name: self.name.clone(),
        }
    }
    fn run_inference(&self, prompt: &str) -> Result<ModelResponse, String> {
        Ok(ModelResponse {
            text: format!("Local sync output: {}", prompt),
        })
    }
    fn load(&mut self, _m: &Path, _t: &Path) -> Result<(), String> { Ok(()) }
    fn generate_stream(&self, _p: &str, _o: InferenceOptions, _tx: Sender<String>) -> Result<(), String> { Ok(()) }
}

pub struct InferenceOptions {
    pub temperature: f64,
    pub top_p: f64,
    pub max_tokens: usize,
}

pub struct LocalCandleEngine {
    device: Device,
    tokenizer: Option<Arc<Tokenizer>>,
    id: String,
    name: String,
}

impl LocalCandleEngine {
    pub fn new(id: &str, name: &str) -> Self {
        let device = Device::cuda_if_available(0).unwrap_or(Device::Cpu);
        Self {
            device,
            tokenizer: None,
            id: id.to_string(),
            name: name.to_string(),
        }
    }
}

impl ModelAdapter for LocalCandleEngine {
    fn info(&self) -> ModelInfo {
        ModelInfo {
            id: self.id.clone(),
            name: self.name.clone(),
        }
    }

    fn run_inference(&self, prompt: &str) -> Result<ModelResponse, String> {
        Ok(ModelResponse {
            text: format!("Sync output placeholder: {}", prompt)
        })
    }

    fn load(&mut self, _model_path: &Path, tokenizer_path: &Path) -> Result<(), String> {
        let tokenizer = Tokenizer::from_file(tokenizer_path)
            .map_err(|e| format!("Failed to load tokenizer: {}", e))?;
        
        self.tokenizer = Some(Arc::new(tokenizer));
        Ok(())
    }

    fn generate_stream(
        &self,
        prompt: &str,
        options: InferenceOptions,
        tx: Sender<String>,
    ) -> Result<(), String> {
        let tokenizer = self.tokenizer.as_ref()
            .ok_or("Model weights or Tokenizer configuration not loaded yet")?
            .clone();

        let tokens = tokenizer.encode(prompt, true).map_err(|e| e.to_string())?;
        let mut tokens_ids = tokens.get_ids().to_vec();
        let device = self.device.clone();
        
        let mut logits_processor = LogitsProcessor::new(299792458, Some(options.temperature), Some(options.top_p));

        tokio::spawn(async move {
            let mut tokens_generated = 0;
            
            while tokens_generated < options.max_tokens {
                let _input = Tensor::new(&*tokens_ids, &device)
                    .unwrap()
                    .unsqueeze(0)
                    .unwrap();

                let dummy_shape = candle_core::Shape::from(1);
                let next_token = logits_processor.sample(&Tensor::zeros(dummy_shape, candle_core::DType::F32, &device).unwrap()).unwrap_or(0u32); 
                tokens_ids.push(next_token);
                tokens_generated += 1;

                if let Ok(text_fragment) = tokenizer.decode(&[next_token], true) {
                    if tx.send(text_fragment).await.is_err() {
                        break; 
                    }
                }
                
                if next_token == 2 { break; }
            }
        });

        Ok(())
    }
}
