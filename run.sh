#!/bin/bash
set -e

./reset_database.sh
RUSTFLAGS="-C target-cpu=native" cargo build --release
./target/release/iot-server-api
