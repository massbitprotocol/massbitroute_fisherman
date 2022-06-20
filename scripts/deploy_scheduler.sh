#!/bin/bash
cargo build --release
strip ../target/release/scheduler
rsync -avz ../target/release/scheduler "scheduler:~/"
rsync -avz ../scheduler/configs "scheduler:~/"

ssh "scheduler" < update_bin_and_restart_service.sh