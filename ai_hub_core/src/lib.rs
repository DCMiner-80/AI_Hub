pub mod downloader;
use std::sync::Arc;

pub trait ModelAdapter: Send + Sync { 
    fn info(&self) -> ModelInfo; 
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

// Create a real struct that implements our model interface template
pub struct LoadedLocalModel {
    pub metadata: ModelInfo,
}

impl ModelAdapter for LoadedLocalModel {
    fn info(&self) -> ModelInfo {
        self.metadata.clone()
    }
}
