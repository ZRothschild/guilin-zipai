# Guilin Paizi Game Startup Script (PowerShell)

Write-Host "=========================================" -ForegroundColor Cyan
Write-Host "      Guilin Paizi Demo Startup Script" -ForegroundColor Cyan
Write-Host "=========================================" -ForegroundColor Cyan

# Check dependencies
function Check-Dependency {
    param($Name, $Command)
    if (!(Get-Command $Command -ErrorAction SilentlyContinue)) {
        Write-Host "X $Name is not installed. Please install it first." -ForegroundColor Red
        return $false
    }
    return $true
}

if (!(Check-Dependency "Rust" "cargo") -or !(Check-Dependency "Trunk" "trunk")) {
    exit 1
}

Write-Host "V Dependencies check passed." -ForegroundColor Green

# Choice
Write-Host "Select Startup Mode:"
Write-Host "1. Start Server (Port: 8080)"
Write-Host "2. Start Client (Port: 3000)"
Write-Host "3. Start Both (Multi-Window)"
$choice = Read-Host "Enter option (1/2/3)"

if ($choice -eq "1") {
    Write-Host "Starting Server..." -ForegroundColor Yellow
    cd crates/server
    cargo run
}
elseif ($choice -eq "2") {
    Write-Host "Starting Client... (Ensure server is running)" -ForegroundColor Yellow
    trunk serve
}
elseif ($choice -eq "3") {
    Write-Host "Starting both Server and Client in separate windows..." -ForegroundColor Yellow
    
    # Start Server in a new window
    Start-Process powershell -ArgumentList "-NoExit", "-Command", "cd crates/server; cargo run"
    
    Write-Host "Waiting for server to initialize..." -ForegroundColor Cyan
    Start-Sleep -Seconds 3
    
    # Start Client in a new window
    Start-Process powershell -ArgumentList "-NoExit", "-Command", "trunk serve"
    
    Write-Host "Server and Client started in separate windows." -ForegroundColor Green
    Write-Host "Access the game at: http://127.0.0.1:3000" -ForegroundColor Cyan
}
else {
    Write-Host "Exited." -ForegroundColor White
}
