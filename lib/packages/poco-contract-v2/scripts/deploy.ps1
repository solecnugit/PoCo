& $PSScriptRoot\build.ps1

if ($LASTEXITCODE -ne 0) {
    Write-Output ">> Build failed"
    Exit -1
}

# Environment variables
$env:NEAR_MODE = "env"
& $PSScriptRoot\near-env.ps1

# Deploy the contract
Write-Output ">> Deploying contract"

$nearEnv = $env:NEAR_ENV ? $env:NEAR_ENV : "local"
$workspaceDir = $env:WORKSPACE_DIR ? $env:WORKSPACE_DIR : $pwd

if ($nearEnv -eq "local") {
    Write-Output ">> Deploying to local network"

    $deployCommand = "pnpm exec near dev-deploy"
} elseif ($nearEnv -eq "testnet") {
    Write-Output ">> Deploying to testnet"

    $deployCommand = "pnpm exec near deploy"
} else {
    Write-Output ">> Unsupported NEAR_ENV: $nearEnv"

    Exit -1
}

"$deployCommand --wasmFile ${workspaceDir}/contract/target/near/poco.wasm --accountId ${$env:NEAR_HELPER_ACCOUNT} -f" | Invoke-Expression

if ($LASTEXITCODE -ne 0) {
    Write-Output ">> Deploy failed"
    Exit -1
}

if (Get-Item $workspaceDir\neardev\dev-account) {
    $contractId = Get-Content $workspaceDir\neardev\dev-account

    Write-Output ">> Contract ID: $contractId"
    Write-Output ">> Deployed."
} else {
    Write-Output ">> Deploy failed"
    Exit -1
}
