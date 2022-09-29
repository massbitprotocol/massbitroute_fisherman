#!/bin/bash

export DOMAIN=massbitroute.net
export RUST_LOG=debug
export SCHEDULER_ENDPOINT=https://scheduler.fisherman.$DOMAIN
export WORKER_IP=$(curl ifconfig.me)
export ZONE={{ZONE}}
export WORKER_ID="$ZONE-$WORKER_IP"
export WORKER_ENDPOINT=http://$WORKER_IP:4040
export WORKER_SERVICE_ENDPOINT=0.0.0.0:4040
export BENCHMARK_WRK_PATH=/opt/fisherman/benchmark
export COMMON_CONFIG_FILE=/opt/fisherman/common.json
export CONFIG_DIR=/opt/fisherman/
cd /opt/fisherman
./fisherman