#!/bin/bash
cargo build --release

if [ -z "$1" ]
then
    #ZONES=( 'as' 'eu' 'na' 'sa' 'oc' 'af' )
    ZONES=( 'as' 'eu' 'na' 'oc' )
else
    ZONES=( "$1" )
fi

for ZN in "${ZONES[@]}"
do
  echo "Create fisherman folder $ZN"
  ssh "worker-demo-$ZN" "mkdir fisherman"

  echo "Update bin file in zone $ZN"
  strip ../target/release/fisherman
  rsync -avz ../target/release/fisherman "worker-demo-$ZN:~/fisherman/fisherman"
  rsync -avz ./run_worker.sh "worker-demo-$ZN:~/fisherman/run.sh"
  rsync -avz ./benchmark "worker-demo-$ZN:~/fisherman/"

  echo "Restart tmux session in zone $ZN"
  ssh "worker-demo-$ZN" < update_bin_and_restart_service_worker.sh
done
