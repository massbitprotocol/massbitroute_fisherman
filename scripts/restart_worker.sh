#!/bin/bash
cargo build --release

if [ -z "$1" ]
then
    #ZONES=( 'as' 'eu' 'na' 'sa' 'oc' 'af' )
    ZONES=( 'as-1' 'as-2' 'eu-2' 'na-2' 'na-3' 'na-4' 'oc-1' )
else
    ZONES=( "$1" )
fi

for ZN in "${ZONES[@]}"
do
  echo "Restart worker in $ZN"
  ssh "worker-demo-$ZN" < script_restart_service_worker.sh
done
