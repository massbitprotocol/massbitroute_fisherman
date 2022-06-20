#!/bin/bash
cargo build --release
strip ../target/release/fisherman
cp ../target/release/fisherman ../../massbitroute_gateway/services/fisherman/
