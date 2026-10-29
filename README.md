# 💻 AI Hub - Desktop Model Downloader & Terminal

AI Hub is a native Windows desktop application built in Rust using **egui**, serving as an interactive interface and downloader for LLMs with multi-threaded Tokio runtimes and Hugging Face's **Candle** ecosystem integration.

---

## 🏗 Project Architecture & Workspace Layout
* **`ai_hub_app/`** - Primary binary crate containing the `egui` GUI layer and terminal shell.
* **`ai_hub_core/`** - Backend library crate with Tokio async download schedulers and verification mechanics.

## 🛠 Installation Summary
1. Install **Visual Studio Build Tools** with "Desktop development with C++" workload (MSVC v143 and Windows SDK).
2. Configure the **Rust Toolchain** via `rustup`.
3. Clone, compile, and execute the workspace using `cargo build --release`.

For the full detailed instructions, architecture primitives, and terminal guide, please refer to the complete documentation block in the repository.
