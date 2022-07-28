#!/bin/bash

if [ -z "$1" ]
then
    #ZONES=( 'as-1' 'as-2' 'eu-2' 'na-2' 'na-3' 'na-4' 'oc-1' )
    ZONES=( 'as-2' 'na-3' 'na-4' )
else
    ZONES=( "$1" )
fi

for ZN in "${ZONES[@]}"
do
  echo "Restart worker in $ZN"
  ssh "worker-demo-$ZN" < script_stop_service_worker.sh
done
