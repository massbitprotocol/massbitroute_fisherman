#!/bin/bash

export DOMAIN=massbitroute.net
export SCHEDULER_ENDPOINT=http://scheduler.fisherman.$DOMAIN
export WORKER_IP=$(curl ifconfig.me)
export ZONE={{ZONE}}
export WORKER_ID="$ZONE-$WORKER_IP"
export WORKER_ENDPOINT=http://$WORKER_IP:4040
export WORKER_SERVICE_ENDPOINT=0.0.0.0:4040
export SCHEDULER_AUTHORIZATION=11967d5e9addc5416ea9224eee0e91fc

cd /opt/fisherman
./fisherman