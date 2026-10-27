# AI Hub - Desktop Model Downloader & Terminal

A native, multi-threaded Windows desktop interface application built in Rust utilizing the **egui** graphical user interface framework. This system features a robust parallel chunked download manager with HTTP range request parsing, atomic background task scheduling, and an interactive local text generation terminal with typewriter live-streaming synchronization.

## 🚀 Key Features

* **⚡ Resumable Parallel Downloader:** Splits target files into proportional segments across multi-threaded background workers to accelerate downloads over network CDNs.
* **🛠️ Slidable Download Manager Workspace:** Custom toggle checkbox interface in the sidebar allows you to fold the download hub overlay in and out of active layout space.
* **💻 Interactive Local Terminal:** Fully interactive single-line prompt input field with a scroll-locked terminal history trace log.
* **✨ Live Response Mocking:** Streams text execution responses character-by-character using frame repaint timers.
* **📦 Verification Gatekeepers:** Automatically validates file paths to verify physical model weights binaries are located on disk before executing prompts.
* **🎨 Dark Cyberpunk Aesthetic:** Tailored deep onyx panels styled with bright neon-green matrix terminal fonts.

---

## 🛠️ Installation & Setup

To compile and launch this native Windows application locally from source code, follow these layout procedures:

### 1. Prerequisites
Ensure you have the Rust toolchain and Microsoft C++ build tools installed on your Windows machine:
* Download and initialize the installer from [rustup.rs](https://rustup.rs).
* Ensure the **Desktop development with C++** workload is checked in the Visual Studio Build Tools window.

### 2. Run the Development Server
To watch your source folder structure, monitor files, and automatically build/re-run your binary on every save file modification event, invoke our hot-reloader automation macro script:
```powershell
.\dev-runner.ps1
```

### 3. Build optimized Production Binaries
To generate a standalone, optimized release executable without debugging symbols trailing behind the framework context:
```powershell
cargo build --release
```
The application binary will be written directly to your local file path target directory:
`.\target\release\ai_hub_app.exe`

---

## 🗂️ Workspace Architecture

* `ai_hub_core/` - Underlying core library managing the `download_with_resume` range pipelines and `ModelAdapter` trait definitions.
* `ai_hub_app/` - Graphical user interface framework wrapper orchestrating state context layers (`ui.rs`), queues (`download_scheduler.rs`), and the main thread loops (`main.rs`).
* `packaging/` - Dynamic metadata manifest properties trackers managing server file indexes.

---

## 🛡️ License
Distributed under the MIT License. See `LICENSE` for more information.
