#!/bin/bash
cargo build --release
strip ../../target/release/scheduler
rsync -avz ../../target/release/scheduler "scheduler:~/"
rsync -avz ../../scheduler/configs "scheduler:~/"
rsync -avz ./run_scheduler.sh "scheduler:~/run.sh"
rsync -avz ../../scheduler/.env_deploy "scheduler:~/.env"

ssh "scheduler" < update_bin_and_restart_scheduler.sh