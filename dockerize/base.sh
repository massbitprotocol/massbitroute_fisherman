#!/bin/bash
export RUNTIME_DIR=/home/huy/work/test_runtime

source .env.${TEST_ENV:-local}

export PROTOCOL=http
export SCHEDULER_AUTHORIZATION=SomeSecretString

export PROXY_IP=254
export TEST_CLIENT_IP=253
export CHAIN_IP=20

export PORTAL_IP=10
export WEB_IP=11
export STAKING_IP=12
export POSTGRES_IP=13
export REDIS_IP=14
export FISHERMAN_SCHEDULER_IP=15
export FISHERMAN_WORKER01_IP=16
export FISHERMAN_WORKER02_IP=17

export GWMAN_IP=2
export GIT_IP=5
export API_IP=6
export SESSION_IP=7

export START_IP=20

export MONITOR_IP=50

ENV=$network_number
export ENV_DIR=$RUNTIME_DIR/$ENV

export domain=massbitroute.net
