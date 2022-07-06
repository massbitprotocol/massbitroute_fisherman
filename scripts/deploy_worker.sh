#!/bin/bash
cargo build --release
strip ../target/release/fisherman
rsync -avz ../target/release/fisherman "demo-gateway-2:~/fisherman/fisherman"
rsync -avz ./run_worker.sh "demo-gateway-2:~/fisherman/run.sh"
rsync -avz ./benchmark "demo-gateway-2:~/fisherman/"

#ssh "demo-gateway-1" < update_bin_and_restart_service_worker.sh