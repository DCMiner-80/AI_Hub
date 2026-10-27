# Force close instances before launching hot reloads to avoid os error 5 locks
Stop-Process -Name "ai_hub_app" -Force -ErrorAction SilentlyContinue
Write-Host "⚡ Watching folder changes. Saving files will trigger automatic recompiles..." -ForegroundColor Cyan
cargo watch -x "run --release"
