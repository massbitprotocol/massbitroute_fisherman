#!/bin/bash
cargo build --release

if [ -z "$1" ]
then
    #ZONES=( 'as-1' 'as-2' 'eu-2' 'na-2' 'na-3' 'na-4' 'oc-1' )
    ZONES=( 'as-1' 'eu-2' 'na-2' 'oc-1' )
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
  ssh "worker-demo-$ZN" < script_restart_service_worker.sh
done
