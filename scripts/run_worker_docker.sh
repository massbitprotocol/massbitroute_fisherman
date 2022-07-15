#!/bin/bash

export ROOT=/massbit/massbitroute/app/src/sites/services/gateway/
export DOMAIN=massbitroute.net
export RUST_LOG=info
export SCHEDULER_ENDPOINT=http://scheduler.fisherman.$DOMAIN
export WORKER_ID=default_worker_id
export WORKER_IP=172.24.24.204
export ZONE=AS
export WORKER_ENDPOINT=http://$WORKER_IP:4040
export WORKER_SERVICE_ENDPOINT=0.0.0.0:4040
export BENCHMARK_WRK_PATH=benchmark

/usr/local/bin/fisherman