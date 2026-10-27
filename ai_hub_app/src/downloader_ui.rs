use std::path::PathBuf;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use ai_hub_core::downloader::{ModelsManifest, download_and_extract_model};

// --- ENUM PERFECTLY MATCHED TO THE EXHAUSTIVE MATCH PATTERNS IN UI_DOWNLOADS.RS ---
pub enum DownloadMsg {
    Progress { id: String, downloaded: u64, total: u64 },
    Done { id: String, ok: Result<(), String> },
}

// --- MATCH YOUR ORIGINAL SCHEDULER SIGNATURE ---
pub fn spawn_download(
    manifest_path: PathBuf,
    model_id: String,
    dest_dir: PathBuf,
    tx: Sender<DownloadMsg>,
) {
    let model_id_clone = model_id.clone();
    let tx_progress = tx.clone();
    let tx_done = tx.clone();

    std::thread::spawn(move || {
        let manifest = match ModelsManifest::from_file(&manifest_path) {
            Ok(m) => m,
            Err(e) => {
                let _ = tx_done.send(DownloadMsg::Done {
                    id: model_id_clone,
                    ok: Err(format!("Manifest failure: {}", e)),
                });
                return;
            }
        };

        let id_cb = model_id_clone.clone();
        let cb = move |downloaded: u64, total: u64| {
            let _ = tx_progress.send(DownloadMsg::Progress {
                id: id_cb.clone(),
                downloaded,
                total,
            });
        };

        match download_and_extract_model(&manifest, &model_id_clone, &dest_dir, Arc::new(cb)) {
            Ok(_) => {
                let _ = tx_done.send(DownloadMsg::Done { id: model_id_clone, ok: Ok(()) });
            }
            Err(err) => {
                let _ = tx_done.send(DownloadMsg::Done { 
                    id: model_id_clone, 
                    ok: Err(err) 
                });
            }
        }
    });
}
