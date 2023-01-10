Write-Output ">> Building contract"

$dir = $pwd
$workspaceDir = $env:WORKSPACE_DIR ? $env:WORKSPACE_DIR : $pwd

Set-Location $workspaceDir\contract

# Install dependencies
rustup target add wasm32-unknown-unknown

# Build the contract
cargo build --target wasm32-unknown-unknown --release

# Copy output
Copy-Item $workspaceDir\contract\target\wasm32-unknown-unknown\release\poco.wasm $workspaceDir\contract\target\near\poco.wasm

Set-Location $dir