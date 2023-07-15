#!/usr/bin/env bash
set -o errexit -o nounset -o pipefail

# set current working directory to script directory to run script from everywhere
cd "$(dirname "$0")"

# This script builds all subprojects and puts all created Wasm modules in one dir
echo "compiling crypto..."
cd crypto
cargo update --aggressive
marine build --release

echo "compiling ipfsdag..."
cd ../ipfsdag
cargo update --aggressive
marine build --release

echo "compiling contract..."
cd ../contract
cargo update --aggressive
marine build --release

echo "compiling node..."
cd ../node
cargo update --aggressive
marine build --release

cd ..
mkdir -p artifacts
rm -f artifacts/*.wasm
cp target/wasm32-wasi/release/crypto.wasm artifacts/
cp target/wasm32-wasi/release/crypto.wasm ../builtin-package/
cp target/wasm32-wasi/release/ipfsdag.wasm artifacts/
cp target/wasm32-wasi/release/ipfsdag.wasm ../builtin-package/
cp target/wasm32-wasi/release/node.wasm artifacts/
cp target/wasm32-wasi/release/node.wasm ../builtin-package/
cp target/wasm32-wasi/release/contract.wasm artifacts/
cp target/wasm32-wasi/release/contract.wasm ../builtin-package/
marine aqua artifacts/node.wasm -s Node -i transaction > ../aqua/node.aqua

wget https://github.com/fluencelabs/sqlite/releases/download/sqlite-wasm-v0.18.1/sqlite3.wasm
mv sqlite3.wasm artifacts/

RUST_LOG="info" mrepl --quiet Config.toml