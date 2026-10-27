use std::path::PathBuf;
use std::sync::mpsc::Sender;
use ai_hub_core::downloader::{ModelsManifest, download_and_extract_model, list_models, ModelEntry};
use anyhow::Result;
use std::thread;
use std::sync::Arc;

#[derive(Debug, Clone)]
pub enum DownloadMsg {
    Progress { id: String, downloaded: u64, total: u64 },
    Done { id: String, ok: Result<(), String> },
}

pub fn spawn_download(manifest_path: PathBuf, model_id: String, dest_dir: PathBuf, tx: Sender<DownloadMsg>) {
    thread::spawn(move || {
        let manifest = match ModelsManifest::from_file(&manifest_path) {
            Ok(m) => m,
            Err(e) => {
                let _ = tx.send(DownloadMsg::Done { id: model_id.clone(), ok: Err(e.to_string()) });
                return;
            }
        };
        let tx_clone = tx.clone();
        let id_clone = model_id.clone();
        let cb = move |downloaded: u64, total: u64| {
            let _ = tx_clone.send(DownloadMsg::Progress { id: id_clone.clone(), downloaded, total });
        };
        let res = download_and_extract_model(&manifest, &model_id, &dest_dir, Arc::new(cb)).map_err(|e| e.to_string());
        let _ = tx.send(DownloadMsg::Done { id: model_id.clone(), ok: res });
    });
}
