use anyhow::{Context, Result};
use reqwest::blocking::Client;
use reqwest::header::RANGE;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use zip::ZipArchive;
use serde::Deserialize;
use std::cmp::min;

#[derive(Debug, Deserialize, Clone)]
pub struct ModelEntry {
    pub id: String,
    pub name: String,
    pub url: String,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ModelsManifest {
    pub models: Vec<ModelEntry>,
}

impl ModelsManifest {
    pub fn from_file<P: AsRef<Path>>(p: P) -> Result<Self> {
        let s = std::fs::read_to_string(p)?;
        let m: ModelsManifest = serde_json::from_str(&s)?;
        Ok(m)
    }
}

pub type ProgressCb = dyn Fn(u64, u64) + Send + Sync + 'static;

fn download_with_resume(client: &Client, url: &str, tmp_path: &Path, total_size: u64, progress_cb: Arc<ProgressCb>, concurrency: usize) -> Result<()> {
    let head = client.head(url).send().context("HEAD request failed")?;
    let accept_ranges = head.headers().get("accept-ranges").map(|v| v.to_str().unwrap_or("").to_lowercase()).unwrap_or_default();
    let supports_range = accept_ranges.contains("bytes") || head.status().is_success();

    if supports_range && total_size > 4 * 1024 * 1024 && concurrency > 1 {
        let f = OpenOptions::new().create(true).write(true).open(tmp_path)?;
        f.set_len(total_size)?;
        drop(f);

        let chunk_size = (total_size as usize + concurrency - 1) / concurrency;
        let client = client.clone();
        let tmp_path = tmp_path.to_path_buf();
        let progress = Arc::new(Mutex::new(0u64));

        let mut handles = Vec::new();
        for i in 0..concurrency {
            let start = (i * chunk_size) as u64;
            let end = min(((i + 1) * chunk_size) as u64 - 1, total_size - 1);
            if start > end { continue; }
            let client = client.clone();
            let tmp_path = tmp_path.clone();
            let progress = progress.clone();
            let progress_cb = progress_cb.clone();
            let url = url.to_string();
            let handle = thread::spawn(move || -> Result<()> {
                let range_header = format!("bytes={}-{}", start, end);
                let mut resp = client.get(&url).header(RANGE, range_header).send().context("range request failed")?;
                let mut out = OpenOptions::new().write(true).open(&tmp_path)?;
                out.seek(SeekFrom::Start(start))?;
                let mut buf = [0u8; 8192];
                loop {
                    let n = resp.read(&mut buf)?;
                    if n == 0 { break; }
                    out.write_all(&buf[..n])?;
                    let mut g = progress.lock().unwrap();
                    *g += n as u64;
                    (progress_cb)(*g, total_size);
                }
                Ok(())
            });
            handles.push(handle);
        }
        for h in handles {
            let r = h.join().map_err(|_| anyhow::anyhow!("thread join error"))?;
            r?;
        }
        Ok(())
    } else {
        let mut downloaded = 0u64;
        if tmp_path.exists() {
            downloaded = tmp_path.metadata()?.len();
        }
        let mut req = client.get(url);
        if downloaded > 0 {
            let range = format!("bytes={}-", downloaded);
            req = req.header(RANGE, range);
        }
        let mut resp = req.send().context("download request failed")?;
        let mut out = OpenOptions::new().create(true).append(true).open(tmp_path)?;
        let mut buf = [0u8; 8192];
        loop {
            let n = resp.read(&mut buf)?;
            if n == 0 { break; }
            out.write_all(&buf[..n])?;
            downloaded += n as u64;
            (progress_cb)(downloaded, total_size);
        }
        Ok(())
    }
}

pub fn download_and_extract_model(manifest: &ModelsManifest, model_id: &str, dest_dir: &Path, progress_cb: Arc<ProgressCb>) -> Result<()> {
    let entry = manifest.models.iter().find(|m| m.id == model_id).context("model id not found in manifest")?;
    let client = Client::builder().timeout(Duration::from_secs(600)).build()?;
    fs::create_dir_all(dest_dir)?;
    let tmp = dest_dir.join(format!("{}.download", entry.id));
    let total_size = entry.size_bytes;
    download_with_resume(&client, &entry.url, &tmp, total_size, progress_cb, 4)?;

    let zipf = File::open(&tmp)?;
    if let Ok(mut archive) = ZipArchive::new(zipf) {
        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let outpath = dest_dir.join(file.mangled_name());
            if (&*file.name()).ends_with('/') {
                fs::create_dir_all(&outpath)?;
            } else {
                if let Some(p) = outpath.parent() { fs::create_dir_all(p)?; }
                let mut outfile = File::create(&outpath)?;
                io::copy(&mut file, &mut outfile)?;
            }
        }
        fs::remove_file(&tmp).ok();
    } else {
        let final_destination = dest_dir.join(format!("{}.bin", entry.id));
        fs::rename(&tmp, final_destination)?;
    }
    Ok(())
}

pub fn list_models(manifest: &ModelsManifest) -> Vec<ModelEntry> {
    manifest.models.clone()
}
