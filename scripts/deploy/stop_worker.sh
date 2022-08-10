#!/bin/bash

if [ -z "$1" ]
then
    #ZONES=( 'as-1' 'as-2' 'eu-1' 'eu-2' 'na-1' 'na-2' 'na-3' 'na-4' 'oc-1' 'oc-2' 'sa-1')
    ZONES=( 'as' 'eu' 'na' 'oc' 'sa')
else
    ZONES=( "$1" )
fi

for ZN in "${ZONES[@]}"
do
  echo "Stop worker in $ZN"
  ssh "worker-demo-$ZN" < script_stop_service_worker.sh
done
