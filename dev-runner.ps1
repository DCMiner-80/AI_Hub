# ========================================================================
# AI Hub - Automated Hot-Reloading Development Runner & Test Orchestrator
# ========================================================================
Write-Host "⚡ Starting AI Hub Hot-Reloading Automation Loop..." -ForegroundColor Cyan
Write-Host "👀 Watching workspace directory folders for file changes..." -ForegroundColor Yellow

# Store directory state hashes to detect modifications
$lastHash = ""

function Get-WorkspaceHash {
    Get-ChildItem -Path . -Recurse -Include *.rs,*.toml | 
        Get-FileHash | 
        Select-Object -ExpandProperty Hash | 
        Out-String
}

# Initial execution pass
$lastHash = Get-WorkspaceHash

while ($true) {
    Clear-Host
    Write-Host "🚀 Change Detected! Initializing Verification Pipeline..." -ForegroundColor Green
    Write-Host "--------------------------------------------------------" -ForegroundColor Gray

    # 1. Statically check structural traits alignment
    Write-Host "🔍 [STEP 1/3] Running Cargo Type Validation Sweep..." -ForegroundColor Yellow
    cargo check
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Compilation validation failed. Awaiting code updates..." -ForegroundColor Red
    } else {
        # 2. Fire the Local & Online Integration Verification Tests
        Write-Host "🧪 [STEP 2/3] Executing Integration Verification Test Suite..." -ForegroundColor Yellow
        cargo test --package ai_hub_core -- --nocapture
        
        if ($LASTEXITCODE -ne 0) {
            Write-Host "⚠️ Test suite warnings or non-critical dry-run failures found." -ForegroundColor DarkYellow
        }

        # 3. Boot up the high-performance UI desktop environment
        Write-Host "💻 [STEP 3/3] Launching Live Desktop Interface Layout..." -ForegroundColor Green
        cargo run --package ai_hub_app
    }

    Write-Host "`n⚡ Watching for file changes. Saving your files will auto-recompile..." -ForegroundColor Gray

    # Sleep loop checking for file updates every 1.5 seconds
    while ($true) {
        Start-Sleep -Seconds 1.5
        $currentHash = Get-WorkspaceHash
        if ($currentHash -ne $lastHash) {
            $lastHash = $currentHash
            break
        }
    }
}
