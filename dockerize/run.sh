#!/bin/bash

if [ ${RUNTIME_MODE} == 'SCHEDULER' ]
then
  echo "RUNTIME_MODE is SCHEDULER"
  cp /usr/local/bin/stats/stats.conf /etc/supervisor/conf.d/
  cp /usr/local/bin/scheduler/scheduler.conf /etc/supervisor/conf.d/
fi

if [ ${RUNTIME_MODE} == 'FISHERMAN' ]
then
  echo "RUNTIME_MODE is FISHERMAN"
  cp /usr/local/bin/worker/fisherman_worker.conf /etc/supervisor/conf.d/
fi

echo "Run supervisord"
supervisord -n