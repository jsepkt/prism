# Prism Network - Multi-Node Local Mesh Cluster Launcher
# Spawns 3 independent validator nodes on localhost connected via P2P wire protocol

Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "    LAUNCHING PRISM NETWORK 3-NODE P2P CLUSTER DEVNET    " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan

# Ensure node binary is built
Write-Host "[Build] Compiling latest prism-node binary..." -ForegroundColor Yellow
cargo build -p prism-node
if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed! Exiting." -ForegroundColor Red
    exit 1
}

$binPath = ".\target\debug\prism-node.exe"

# Node 1: Genesis Bootstrap Validator (HTTP: 8545, P2P: 9000)
Write-Host "`n[Node 1] Spawning Genesis Validator on HTTP: 8545 | P2P: 9000..." -ForegroundColor Green
$p1 = Start-Process -FilePath $binPath -Environment @{
    "PRISM_PORT" = "8545"
    "PRISM_P2P_PORT" = "9000"
} -PassThru

Start-Sleep -Seconds 2

# Node 2: Peer Node (HTTP: 8546, P2P: 9001 -> Connects to Node 1)
Write-Host "[Node 2] Spawning Peer Validator on HTTP: 8546 | P2P: 9001..." -ForegroundColor Green
$p2 = Start-Process -FilePath $binPath -Environment @{
    "PRISM_PORT" = "8546"
    "PRISM_P2P_PORT" = "9001"
    "PRISM_BOOTSTRAP_PEERS" = "127.0.0.1:9000"
} -PassThru

Start-Sleep -Seconds 1

# Node 3: Peer Node (HTTP: 8547, P2P: 9002 -> Connects to Node 1)
Write-Host "[Node 3] Spawning Peer Validator on HTTP: 8547 | P2P: 9002..." -ForegroundColor Green
$p3 = Start-Process -FilePath $binPath -Environment @{
    "PRISM_PORT" = "8547"
    "PRISM_P2P_PORT" = "9002"
    "PRISM_BOOTSTRAP_PEERS" = "127.0.0.1:9000"
} -PassThru

Write-Host "`n==========================================================" -ForegroundColor Cyan
Write-Host "             CLUSTER RUNNING SUCCESSFULLY!                " -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "  - Node 1 Dashboard: http://127.0.0.1:8545" -ForegroundColor White
Write-Host "  - Node 2 RPC API:   http://127.0.0.1:8546" -ForegroundColor White
Write-Host "  - Node 3 RPC API:   http://127.0.0.1:8547" -ForegroundColor White
Write-Host "`nPress Ctrl+C or close the terminal to stop the cluster." -ForegroundColor Gray

# Wait on processes
try {
    Wait-Process -Id $p1.Id, $p2.Id, $p3.Id
} finally {
    Write-Host "`nTerminating cluster nodes..." -ForegroundColor Yellow
    Stop-Process -Id $p1.Id, $p2.Id, $p3.Id -ErrorAction SilentlyContinue
}
