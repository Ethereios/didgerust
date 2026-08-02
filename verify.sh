#!/usr/bin/env bash
# CADSD Verification Script
# Run: bash verify.sh

echo "=== CADSD Verification Script ==="
echo ""

echo "1. Checking Rust version..."
rustc --version

echo ""
echo "2. Running core library tests..."
cargo test --release --lib --test-threads=4

echo ""
echo "3. Running integration tests..."
cargo test --release --test integration_tests --test-threads=4

echo ""
echo "4. Building core library..."
cargo build --release

echo ""
echo "5. Building GUI binary..."
cargo build --release --bin cadsd-gui --features gui-bevy

echo ""
echo "=== Verification Complete ==="
echo "Launch GUI: cargo run --release --bin cadsd-gui --features gui-bevy"