$mode = $env:NEAR_MODE ? $env:NEAR_MODE : "unset"

$netowrkIp = $env:NEAR_NETWORK_IP ? $env:NEAR_NETWORK_IP : "49.52.27.50"
$workspaceDir = $env:WORKSPACE_DIR ? $env:WORKSPACE_DIR : $pwd

if ($mode -eq "env") {
    Write-Output ">> Setting NEAR environment variables"

    $env:NEAR_ENV="local"
    $env:NEAR_CLI_LOCALNET_NETWORK_ID="localnet"
    $env:NEAR_NODE_URL="http://${netowrkIp}:8332"
    $env:NEAR_CLI_LOCALNET_KEY_PATH="${workspaceDir}/validator-key.json"
    $env:NEAR_WALLET_URL="http://${netowrkIp}:8334"
    $env:NEAR_HELPER_URL="http://${netowrkIp}:8330"
    $env:NEAR_HELPER_ACCOUNT="test.near"
    $env:NEAR_EXPLORER_URL="http://${netowrkIp}:8331"

    if (Get-Item $workspaceDir\neardev\dev-account) {
        $contractId = Get-Content $workspaceDir\neardev\dev-account

        $env:CONTRACT_ID = $contractId
    } else {
        Remove-Item Env:\CONTRACT_ID -ErrorAction SilentlyContinue
    }
} elseif ($mode -eq "unset") {
    Write-Output ">> Clearing NEAR environment variables"

    Remove-Item Env:\NEAR_ENV -ErrorAction SilentlyContinue
    Remove-Item Env:\NEAR_CLI_LOCALNET_NETWORK_ID -ErrorAction SilentlyContinue
    Remove-Item Env:\NEAR_NODE_URL -ErrorAction SilentlyContinue
    Remove-Item Env:\NEAR_CLI_LOCALNET_KEY_PATH -ErrorAction SilentlyContinue
    Remove-Item Env:\NEAR_WALLET_URL -ErrorAction SilentlyContinue
    Remove-Item Env:\NEAR_HELPER_URL -ErrorAction SilentlyContinue 
    Remove-Item Env:\NEAR_HELPER_ACCOUNT -ErrorAction SilentlyContinue
    Remove-Item Env:\NEAR_EXPLORER_URL -ErrorAction SilentlyContinue 
    Remove-Item Env:\NEAR_DEVELOPMENT_COMMAND -ErrorAction SilentlyContinue

    Remove-Item Env:\CONTRACT_ID -ErrorAction SilentlyContinue
}