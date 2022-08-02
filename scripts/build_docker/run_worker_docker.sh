#!/bin/bash

if [ $ROLE = 'SCHEDULER' ]
then
  supervisorctl stop fisherman
  exit
fi

#export id="$(echo $RANDOM | md5sum | head -c 5)"
export ROOT=/massbit/massbitroute/app/src/sites/services/gateway/
export DOMAIN=massbitroute.net
export RUST_LOG=debug
export SCHEDULER_ENDPOINT=http://scheduler.fisherman.$DOMAIN:80
export WORKER_ID=default_worker_$WORKER_IP
export ZONE=AS
export WORKER_ENDPOINT=http://$WORKER_IP:4040
export WORKER_SERVICE_ENDPOINT=0.0.0.0:4040
export BENCHMARK_WRK_PATH=benchmark

/usr/local/bin/fisherman