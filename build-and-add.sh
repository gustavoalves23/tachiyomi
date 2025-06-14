#!/bin/bash
set -e

cargo build --manifest-path ./keiyoushi-clone/Cargo.toml --release
mkdir -p ./bin
cp ./keiyoushi-clone/target/release/keiyoushi-clone ./bin/
git add ./bin/keiyoushi-clone
echo "Build done and binary staged for commit."

