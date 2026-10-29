# 💻 AI Hub - Desktop Model Downloader & Terminal

AI Hub is a highly optimized, native Windows desktop application built in Rust using the **egui** framework. It acts as an interactive bridge to open-source LLMs, featuring multi-threaded chunked download pipelines, background asynchronous task workers, local file integrity gates, and direct integration with **Candle** for GPU/hardware acceleration.

---

## 🚀 Core Architectural Primitives

- **⚡ Fluid Asynchronous Runtime:** Built on top of a multi-threaded **Tokio** thread pool. Heavy network requests and heavy background weights are isolated from the main thread, keeping the immediate-mode GUI processing at a crisp 60 FPS.
- **📥 Resumable Chunked Downloader:** Orchestrates HTTP range-request channels to break deep learning binaries down into chunks. Downloads occur concurrently over safe memory partitions.
- **🛡 Validation Gatekeeper:** Runs rapid local diagnostics to check file sizes and missing boundaries before registering model files locally.
- **🔌 Model Context Protocol (MCP):** Exposes active runtime registries, permitting integration with Telegram/Discord network broadcasts.

---

## 🛠️ Step-by-Step Installation & Prerequisites

To build and run AI Hub from source, you must configure a native C++ compile platform and a stable Rust environment toolchain layout.

### Step 1: Install Visual Studio Build Tools (C++ Compiler)
The GPU libraries and desktop rendering systems require the native MSVC compiler tools.
1. Download the **[Visual Studio Installer](https://microsoft.com)**.
2. Run the installer and check the box for **Desktop development with C++**.
3. Ensure the following individual components are included on the right-hand panel:
   - **MSVC v143 - VS 2022 C++ x64/x86 build tools**
   - **Windows 11 SDK** (or Windows 10 SDK depending on your OS)
4. Click Install and restart your computer if requested.

### Step 2: Install the Rust Toolchain Environment
1. Download and execute **[rustup-init.exe](https://rustup.rs)**.
2. When prompted in the terminal panel, press **1** to proceed with the standard system installation.
3. Once completed, reload your PowerShell terminal or run `refreshenv` to verify compiler availability:
   ```powershell
   rustc --version
   cargo --version
   ```

### Step 3: Clone and Compile the Application
Run these commands inside your terminal to clone your repository, switch folders, and trigger an optimized compilation run:

```powershell
# Clone the repository tree
git clone https://github.com
cd AI_Hub

# Compile the stable production binary layout
cargo build --release
```
The optimized executable will be generated at **`.\target\release\ai_hub_app.exe`**.

---

## 🕹️ Interface Commands & Usage Guide

When you launch the application window, you can run the following standard string operations down inside the text shell terminal input box:

- `download <model_id>`: Spawns the concurrent background downloader engine to safely pull down file bytes without blocking frame painting loops.
- `clear`: Flushes the active text console buffer window back to a clean terminal state.