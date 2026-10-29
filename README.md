# AI Hub - Desktop Model Downloader & Terminal

A native Windows desktop application built in Rust using the **egui** framework, featuring parallel chunked downloading, local validation, and an interactive text generation terminal.

---

## 🚀 Core Product Capabilities

- **⚡ Resumable Parallel Downloader:** Multi-threaded background workers via HTTP range requests.
- **🛡️ Fallback Validation Gatekeeper:** Local storage checks for existing binaries.
- **📁 Custom Directory Picker:** Native folder selection via `rfd`.
- **💻 Interactive Matrix Terminal:** Dashboard with trace logging.
- **✨ Live Typewriter Streamer:** Frame-synchronized text streaming.
- **⚙️ GPU Acceleration Engine:** Integrates with **Candle** for weight execution.

---

## 🛠️ Release Changelog & Architecture History

Includes recent updates across versions `v0.2.0` through `v0.4.0`. For detailed setup and workspace architecture, please refer to the repository documentation.

---

## 🛠️ Installation & Setup

1. **Prerequisites:** Install the Rust toolchain and C++ build tools.
2. **Clone & Build:**
   ```bash
   git clone https://github.com
   cd AI_Hub
   cargo build --release
   ```
   Output binary location: `.\target\release\ai_hub_app.exe`.

---

## 🛡️ License

Distributed under the MIT License.
