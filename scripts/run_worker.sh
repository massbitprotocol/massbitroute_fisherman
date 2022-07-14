#!/bin/bash

export ROOT=/massbit/massbitroute/app/src/sites/services/gateway/
export DOMAIN=massbitroute.net
export RUST_LOG=info
export SCHEDULER_ENDPOINT=https://scheduler.fisherman.$DOMAIN
export WORKER_ID=$(cat $ROOT/vars/ID)
export WORKER_IP=$(cat $ROOT/vars/IP)
export ZONE=$(cat $ROOT/vars/RAW | jq .geo.continentCode)
export WORKER_ENDPOINT=https://$WORKER_IP/__worker
export WORKER_SERVICE_ENDPOINT=0.0.0.0:4040
export BENCHMARK_WRK_PATH=benchmark

./fisherman