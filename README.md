# 💻 AI Hub - Desktop Model Downloader & Terminal

AI Hub is a native Windows desktop application built in Rust using **egui**, featuring asynchronous downloads, local validation, and **Candle** GPU integration.

---

## 🚀 Core Features
* **Asynchronous Runtime:** Powered by **Tokio** to keep the GUI responsive at 60 FPS.
* **Resumable Downloads:** HTTP range-request chunks for deep learning binaries.
* **Validation & Serialization:** Local file verification and **Serde** data integration.

## 🏗 Project Layout
* `ai_hub_app/`: Frontend `egui` interface and terminal parsers.
* `ai_hub_core/`: Execution engine, networking, and model backends.

## 🛠 Quick Setup
1. Install **Visual Studio Build Tools** (Desktop development with C++).
2. Install the **Rust toolchain** via `rustup`.
3. Build the project:
   ```powershell
   cargo build --release
   ```
