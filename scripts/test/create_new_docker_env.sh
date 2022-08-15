#!/bin/bash

export FISHERMAN_TAG=current

export STAKING_TAG=v0.1
export PORTAL_TAG=v0.1
export WEB_TAG=v0.1
export CHAIN_TAG=v0.1
export API_TAG=v0.1.4
export GIT_TAG=v0.1.5
export GWMAN_TAG=v0.1.0
export STAT_TAG=v0.1.0
export MONITOR_TAG=v0.1.0

while docker network ls -q | grep "$find_string"
do
    network_number=$(shuf -i 0-255 -n 1)
    find_string="\"Subnet\": \"172.24.$network_number.0/24\","
    echo $find_string
done

echo "--------------------------------------------"
echo "Creating network 172.24.$network_number.0/24"
echo "--------------------------------------------"

docker network create -d bridge --gateway "172.24.$network_number.1" --subnet "172.24.$network_number.0/24"   mbr_test_network_$network_number

echo "--------------------------------------------"
echo "Generating docker compose for core components"
echo "--------------------------------------------"

#TODO: Remove this 
# network_number="24"



    
cat docker-git/docker-compose.yaml.template |  \
	 sed "s/\[\[RUN_ID\]\]/$network_number/g" | \
	 sed "s/\[\[NETWORK_NUMBER\]\]/$network_number/g" | \
     sed "s/\[\[STAKING_TAG\]\]/$STAKING_TAG/g" | \
     sed "s/\[\[PORTAL_TAG\]\]/$PORTAL_TAG/g" | \
     sed "s/\[\[WEB_TAG\]\]/$WEB_TAG/g" | \
     sed "s/\[\[CHAIN_TAG\]\]/$CHAIN_TAG/g" | \
     sed "s/\[\[API_TAG\]\]/$API_TAG/g" | \
     sed "s/\[\[GIT_TAG\]\]/$GIT_TAG/g" | \
     sed "s/\[\[STAKING_TAG\]\]/$STAKING_TAG/g" | \
     sed "s/\[\[GWMAN_TAG\]\]/$GWMAN_TAG/g" | \
     sed "s/\[\[STAT_TAG\]\]/$STAT_TAG/g" | \
     sed "s/\[\[MONITOR_TAG\]\]/$MONITOR_TAG/g" \
    > docker-git/docker-compose.yaml




cd docker-git
docker-compose up -d
docker exec -it mbr_git_$network_number rm -rf /massbit/massbitroute/app/src/sites/services/git/vars/*
docker exec -it mbr_git_$network_number rm -rf /massbit/massbitroute/app/src/sites/services/git/data/*
docker exec -it mbr_git_$network_number /massbit/massbitroute/app/src/sites/services/git/scripts/run _repo_init
PRIVATE_GIT_READ=$(docker exec -it mbr_git_$network_number cat /massbit/massbitroute/app/src/sites/services/git/data/env/git.env  | grep GIT_PRIVATE_READ_URL  | cut -d "=" -f 2 | sed "s/'//g")
echo $PRIVATE_GIT_READ

cat docker-node/docker-compose.yaml.template |  \
	 sed "s/\[\[RUN_ID\]\]/$network_number/g" | \
	 sed "s/\[\[NETWORK_NUMBER\]\]/$network_number/g" | \
     sed "s/\[\[STAKING_TAG\]\]/$STAKING_TAG/g" | \
     sed "s/\[\[PORTAL_TAG\]\]/$PORTAL_TAG/g" | \
     sed "s/\[\[WEB_TAG\]\]/$WEB_TAG/g" | \
     sed "s/\[\[CHAIN_TAG\]\]/$CHAIN_TAG/g" | \
     sed "s/\[\[API_TAG\]\]/$API_TAG/g" | \
     sed "s/\[\[GIT_TAG\]\]/$GIT_TAG/g" | \
     sed "s/\[\[GWMAN_TAG\]\]/$GWMAN_TAG/g" | \
     sed "s/\[\[STAT_TAG\]\]/$STAT_TAG/g" | \
     sed "s|\[\[PRIVATE_GIT_READ\]\]|$PRIVATE_GIT_READ|g" | \
     sed "s/\[\[MONITOR_TAG\]\]/$MONITOR_TAG/g" \
    > docker-node/docker-compose.yaml

cat docker-compose.yaml.template |  \
	 sed "s/\[\[RUN_ID\]\]/$network_number/g" | \
	 sed "s/\[\[NETWORK_NUMBER\]\]/$network_number/g" | \
     sed "s/\[\[STAKING_TAG\]\]/$STAKING_TAG/g" | \
     sed "s/\[\[PORTAL_TAG\]\]/$PORTAL_TAG/g" | \
     sed "s/\[\[WEB_TAG\]\]/$WEB_TAG/g" | \
     sed "s/\[\[CHAIN_TAG\]\]/$CHAIN_TAG/g" | \
     sed "s/\[\[API_TAG\]\]/$API_TAG/g" | \
     sed "s/\[\[GIT_TAG\]\]/$GIT_TAG/g" | \
     sed "s/\[\[GWMAN_TAG\]\]/$GWMAN_TAG/g" | \
     sed "s/\[\[STAT_TAG\]\]/$STAT_TAG/g" | \
     sed "s|\[\[PRIVATE_GIT_READ\]\]|$PRIVATE_GIT_READ|g" | \     
     sed "s/\[\[MONITOR_TAG\]\]/$MONITOR_TAG/g"  \
    > docker-compose.yaml

cat docker-gateway/docker-compose.yaml.template |  \
	 sed "s/\[\[RUN_ID\]\]/$network_number/g" | \
	 sed "s/\[\[NETWORK_NUMBER\]\]/$network_number/g" | \
     sed "s/\[\[STAKING_TAG\]\]/$STAKING_TAG/g" | \
     sed "s/\[\[PORTAL_TAG\]\]/$PORTAL_TAG/g" | \
     sed "s/\[\[WEB_TAG\]\]/$WEB_TAG/g" | \
     sed "s/\[\[CHAIN_TAG\]\]/$CHAIN_TAG/g" | \
     sed "s/\[\[API_TAG\]\]/$API_TAG/g" | \
     sed "s/\[\[GIT_TAG\]\]/$GIT_TAG/g" | \
     sed "s/\[\[GWMAN_TAG\]\]/$GWMAN_TAG/g" | \
     sed "s/\[\[STAT_TAG\]\]/$STAT_TAG/g" | \
     sed "s|\[\[PRIVATE_GIT_READ\]\]|$PRIVATE_GIT_READ|g" | \
     sed "s/\[\[MONITOR_TAG\]\]/$MONITOR_TAG/g"  \
    > docker-gateway/docker-compose.yaml