#!/bin/bash

export RUST_LOG=debug
export RUST_LOG_TYPE=console
export DATABASE_URL=postgres://postgres:postgres@db:5432/massbit-fisherman
export DOMAIN=massbitroute.net
export PORTAL_AUTHORIZATION=g2xnS1uKr4Ko7tPApdceP4NSOKhhbWbX
export REPORT_CALLBACK=http://scheduler.fisherman.$DOMAIN/report
export SCHEDULER_ENDPOINT=0.0.0.0:80
export PATH_GATEWAYS_LIST=mbr/gateway/list/verify
export PATH_NODES_LIST=mbr/node/list/verify
export PATH_PORTAL_PROVIDER_REPORT=mbr/benchmark
export PATH_PORTAL_PROVIDER_VERIFY=mbr/verify
export URL_PORTAL=http://portal.$DOMAIN
export IS_TEST_MODE=false

/usr/local/bin/scheduler
