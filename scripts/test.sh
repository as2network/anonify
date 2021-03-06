#!/bin/bash

set -e

source /root/.docker_bashrc
export PATH=~/.cargo/bin:$PATH
export SGX_MODE=HW
export RUSTFLAGS=-Ctarget-feature=+aes,+sse2,+sse4.1,+ssse3

dirpath=$(cd $(dirname $0) && pwd)
cd "${dirpath}/.."
solc -o contract-build --bin --abi --optimize --overwrite contracts/Anonify.sol

cd frame/types
cargo build

cd ../../scripts
# Generate a `enclave.signed.so` in `$HOME/.anonify`
make DEBUG=1 ENCLAVE_DIR=example/erc20/enclave

# Testings

echo "Integration testing..."
cd ../tests/integration
RUST_BACKTRACE=1 RUST_LOG=debug cargo test -- --nocapture

cd ../../example/erc20/server
RUST_BACKTRACE=1 RUST_LOG=debug cargo test test_deploy_post -- --nocapture
sleep 1
RUST_BACKTRACE=1 RUST_LOG=debug cargo test test_multiple_messages -- --nocapture
sleep 1
RUST_BACKTRACE=1 RUST_LOG=debug cargo test test_skip_invalid_event -- --nocapture
sleep 1
RUST_BACKTRACE=1 RUST_LOG=debug cargo test test_node_recovery -- --nocapture
sleep 1
RUST_BACKTRACE=1 RUST_LOG=debug cargo test test_join_group_then_handshake -- --nocapture

echo "Unit testing..."
cd ../../../scripts
make DEBUG=1 TEST=1 ENCLAVE_DIR=tests/units/enclave
cd ..
RUST_BACKTRACE=1 RUST_LOG=debug cargo test -p unit-tests-host -p anonify-eth-driver -p frame-runtime -- --nocapture

# Buildings

export ANONIFY_URL=http://172.28.1.1:8080
./scripts/build-cli.sh

echo "Building ERC20 server..."
cd example/erc20/server
RUST_BACKTRACE=1 RUST_LOG=debug cargo build
