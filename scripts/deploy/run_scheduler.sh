#!/bin/bash

export CONFIG_DIR=/opt/fisherman/configs/tasks
export DOMAIN=massbitroute.net
export IS_VERIFY_REPORT=true
export IS_REGULAR_REPORT=true
export PATH_GATEWAYS_LIST=mbr/gateway/list/verify
export PATH_NODES_LIST=mbr/node/list/verify
export PATH_PORTAL_PROVIDER_REPORT=mbr/benchmark
export PATH_PORTAL_PROVIDER_VERIFY=mbr/verify
export RUST_LOG=debug
export RUST_LOG_TYPE=console
export REPORT_CALLBACK=https://scheduler.fisherman.$DOMAIN/report
export SCHEDULER_ENDPOINT=0.0.0.0:3031
export SCHEDULER_CONFIG=/opt/fisherman/configs/scheduler.json
export URL_PORTAL=https://portal.$DOMAIN

cd /opt/fisherman
./scheduler
