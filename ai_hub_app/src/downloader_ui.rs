use std::sync::Arc;
use std::path::PathBuf;
use ai_hub_core::downloader::{ModelsManifest, download_and_extract_model};

pub enum DownloadMsg {
    Progress { id: String, downloaded: u64, total: u64 },
    Done { id: String, ok: Result<(), String> },
}

pub fn spawn_download(
    manifest: ModelsManifest,
    model_id: String,
    dest_dir: PathBuf,
    tx: std::sync::mpsc::Sender<DownloadMsg>
) {
    let tx_clone = tx.clone();
    let model_id_clone = model_id.clone();
    
    std::thread::spawn(move || {
        let cb_tx = tx_clone.clone();
        let cb_id = model_id_clone.clone();
        
        let cb = move |downloaded: u64, total: u64| {
            let _ = cb_tx.send(DownloadMsg::Progress {
                id: cb_id.clone(),
                downloaded,
                total,
            });
        };

        let result = download_and_extract_model(&manifest, &model_id_clone, &dest_dir, Arc::new(cb));
        
        let _ = tx_clone.send(DownloadMsg::Done {
            id: model_id_clone,
            ok: result.map(|_| ()),
        });
    });
}
