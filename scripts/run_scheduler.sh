#!/bin/bash

export RUST_LOG=info
export RUST_LOG_TYPE=console
export DATABASE_URL=postgres://fisherman:FishermanCodelight123@35.193.163.173:5432/massbit-fisherman
export DOMAIN=massbitroute.net
export PORTAL_AUTHORIZATION=g2xnS1uKr4Ko7tPApdceP4NSOKhhbWbX
export PATH_GATEWAYS_LIST=mbr/gateway/list/verify
export PATH_NODES_LIST=mbr/node/list/verify
export REPORT_CALLBACK=https://scheduler.fisherman.$DOMAIN/report
export URL_PORTAL=https://portal.$DOMAIN

/opt/fisherman/scheduler
