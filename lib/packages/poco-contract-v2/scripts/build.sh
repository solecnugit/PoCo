#!/bin/sh

echo ">> Building contract"

cd $WORKSPACE_DIR/contract

rustup target add wasm32-unknown-unknown

cargo install cargo-near --vers 0.6.1

echo ">> finished install"

# cargo build --all --target wasm32-unknown-unknown --release

# cargo near build --release
cargo near abi

# cargo near build  --release
cargo build --target wasm32-unknown-unknown --release

echo ">> finished build"

cp $WORKSPACE_DIR/contract/target/near/poco_abi.json $WORKSPACE_DIR/../poco-agent/abi.json