use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::collections::VecDeque;
use std::path::PathBuf;
use crate::downloader_ui::{spawn_download, DownloadMsg};
use std::sync::mpsc::Sender;

pub struct DownloadScheduler {
    queue: Arc<Mutex<VecDeque<String>>>,
    manifest: PathBuf,
    dest_dir: PathBuf,
    tx_ui: Sender<DownloadMsg>,
}

impl DownloadScheduler {
    pub fn new(manifest: PathBuf, dest_dir: PathBuf, tx_ui: Sender<DownloadMsg>) -> Self {
        Self { queue: Arc::new(Mutex::new(VecDeque::new())), manifest, dest_dir, tx_ui }
    }
    pub fn enqueue(&self, model_id: String) {
        let mut q = self.queue.lock().unwrap();
        q.push_back(model_id);
    }
    pub fn start(self: Arc<Self>) {
        let q = self.queue.clone();
        let manifest = self.manifest.clone();
        let dest = self.dest_dir.clone();
        let tx = self.tx_ui.clone();
        thread::spawn(move || {
            loop {
                let next = { let mut g = q.lock().unwrap(); g.pop_front() };
                if let Some(model_id) = next {
                    spawn_download(manifest.clone(), model_id, dest.clone(), tx.clone());
                    thread::sleep(Duration::from_secs(2));
                } else {
                    thread::sleep(Duration::from_secs(1));
                }
            }
        });
    }
}
