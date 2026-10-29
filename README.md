# 💻 AI Hub - Desktop Model Downloader & Terminal

A native Windows desktop application built in Rust using the **egui** framework, featuring parallel chunked downloading and asynchronous task systems.

## 🚀 Core Product Capabilities (v2.0.0 Upgrade)
- **⚡ Asynchronous Tokio Event Runtime:** Ensures a fluid 60 FPS workspace interface with no visual freezing.
- **📥 Parallel Chunked Downloader:** Utilizes HTTP range requests to download model layers via multi-threaded background workers.
- **💻 Integrated Side-by-Side Dashboard:** Dual-column grid layout with real-time download monitoring.

## 📦 Distribution & Usage
Pre-compiled executable archive bundles are built automatically via GitHub Actions. Run `cargo run --release` to build from source.