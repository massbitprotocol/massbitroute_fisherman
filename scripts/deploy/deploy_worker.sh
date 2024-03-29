#!/bin/bash
cargo build --release

if [ -z "$1" ]
then
    #ZONES=( 'as-1' 'as-2' 'eu-2' 'na-2' 'na-3' 'na-4' 'oc-1' )
    #ZONES=( 'as-1' 'eu-2' 'na-2' 'oc-1' 'sa-1')
    ZONES=( 'as' 'eu' 'na' 'oc' 'sa')
else
    ZONES=( "$1" )
fi

strip ../../target/release/fisherman
for ZN in "${ZONES[@]}"
do
  echo "Create fisherman folder $ZN"
  #ssh "worker-demo-$ZN" "rm ~/fisherman -rf"
  ssh "worker-demo-$ZN" "mkdir -p ~/fisherman"

  echo "Update bin file in zone $ZN"
  rsync -avz ../../target/release/fisherman "worker-demo-$ZN:~/fisherman/fisherman"
  # rsync -avz ../benchmark "worker-demo-$ZN:~/fisherman/"
  # rsync -avz ../../.env_fisherman "worker-demo-$ZN:~/fisherman/.env_fisherman"
  # rsync -avz ../../common/configs/common.json "worker-demo-$ZN:~/fisherman/common.json"
  # rsync -avz ../../fisherman/configs/log.yaml "worker-demo-$ZN:~/fisherman/log.yaml"
  rsync -avz ../benchmark "worker-demo-$ZN:~/fisherman/"
  rsync -avz ../../dockerize/worker_config/configs.production "worker-demo-$ZN:~/fisherman/"
  rsync -avz ./fisherman.conf "worker-demo-$ZN:~/fisherman/fisherman.conf"
  rsync -avz ../../dockerize/worker_config/.env.deploy "worker-demo-$ZN:~/fisherman/.env"

  # Create run script
  cat ./run_worker.sh | sed "s/{{ZONE}}/$ZN/g" > _run_worker_"$ZN".sh
  # Copy run script
  rsync -avz ./_run_worker_"$ZN".sh "worker-demo-$ZN:~/fisherman/run.sh"
  # Remove template
  rm _run_worker_"$ZN".sh

  echo "Restart tmux session in zone $ZN"
  ssh "worker-demo-$ZN" < update_bin_and_restart_service_worker.sh
done
