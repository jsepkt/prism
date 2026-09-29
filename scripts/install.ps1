# ==============================================================================
# Prism Network Node: Universal 1-Line Installer (Windows PowerShell)
# Usage: iwr -useb https://raw.githubusercontent.com/jsepkt/prism/main/scripts/install.ps1 | iex
# ==============================================================================

$ErrorActionPreference = "Stop"

Write-Host @"
  _____  _____  _____  _____ __  __   _   _ ______ _______ 
 |  __ \|  __ \|_   _|/ ____|  \/  | | \ | |  ____|__   __|
 | |__) | |__) | | | | (___ | \  / | |  \| | |__     | |   
 |  ___/|  _  /  | |  \___ \| |\/| | | . ` |  __|    | |   
 | |    | | \ \ _| |_ ____) | |  | | | |\  | |____   | |   
 |_|    |_|  \_\_____|_____/|_|  |_| |_| \_|______|  |_|   
      Sovereign Context & Edge-AI Ledger (Layer-1)
"@ -ForegroundColor Magenta

Write-Host "==> Installing Prism Full Node & Validator Client on Windows..." -ForegroundColor Cyan

# 1. Check Git
if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    Write-Error "Git is required. Please install Git for Windows (https://git-scm.com)."
    exit 1
}

# 2. Check Cargo / Rust
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Warning: Rust & Cargo not found. Please install Rust via https://rustup.rs." -ForegroundColor Yellow
    exit 1
}

# 3. Clone or Update
$installDir = "$HOME\.prism"
if (Test-Path $installDir) {
    Write-Host "==> Updating existing Prism repository at $installDir..." -ForegroundColor Cyan
    Set-Location $installDir
    git fetch origin main
    git reset --hard origin/main
} else {
    Write-Host "==> Cloning Prism Network repository into $installDir..." -ForegroundColor Cyan
    git clone https://github.com/jsepkt/prism.git $installDir
    Set-Location $installDir
}

# 4. Compile Release
Write-Host "==> Compiling prism-node with release optimizations..." -ForegroundColor Cyan
cargo build --release -p prism-node

$binSrc = "$installDir\target\release\prism-node.exe"
$localBin = "$HOME\.local\bin"
if (-not (Test-Path $localBin)) {
    New-Item -ItemType Directory -Path $localBin -Force | Out-Null
}
Copy-Item $binSrc -Destination "$localBin\prism-node.exe" -Force

# Ensure ~/.local/bin is on PATH
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$localBin*") {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$localBin", "User")
    $env:Path += ";$localBin"
}

Write-Host "`n✔ Prism Node successfully installed to $localBin\prism-node.exe!`n" -ForegroundColor Green
Write-Host "To start your node:"
Write-Host "  prism-node" -ForegroundColor Cyan
Write-Host ""
Write-Host "Dashboard available at:"
Write-Host "  http://127.0.0.1:8545" -ForegroundColor Magenta
Write-Host ""
Write-Host "Public Web Explorer & Wallet:"
Write-Host "  https://jsepkt.github.io/prism/" -ForegroundColor Cyan
Write-Host ""
