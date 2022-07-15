#!/bin/bash

export RUST_LOG=info
export RUST_LOG_TYPE=console
export DATABASE_URL=postgres://postgres:postgres@db:5432/massbit-fisherman
export DOMAIN=massbitroute.net
export PORTAL_AUTHORIZATION=g2xnS1uKr4Ko7tPApdceP4NSOKhhbWbX
export URL_GATEWAYS_LIST=http://portal.$DOMAIN/mbr/gateway/list/verify
export URL_NODES_LIST=http://portal.$DOMAIN/mbr/node/list/verify
export URL_PORTAL_PROVIDER_REPORT=http://portal.$DOMAIN/mbr/benchmark
export REPORT_CALLBACK=http://scheduler.fisherman.$DOMAIN/report
export SCHEDULER_ENDPOINT=0.0.0.0:80

/usr/local/bin/scheduler
