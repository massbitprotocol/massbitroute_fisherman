#!/bin/bash

export network_number=$1
export FISHER_ENVIRONMENT=local

ROOT_DIR=$(realpath $(dirname $(realpath $0)))
ROOT_DIR_MBR_TEST=/home/huy/work/block_chain/project-massbit-route/massbitroute_test/end2end

OUTPUT_PATH=$ROOT_DIR
TEMPLATE_PATH=$ROOT_DIR

source $ROOT_DIR/base.sh
#Get PRIVATE_GIT_READ
#PRIVATE_GIT_READ=$(docker exec -it mbr_git_$network_number cat /massbit/massbitroute/app/src/sites/services/git/data/env/git.env  | grep GIT_PRIVATE_READ_URL  | cut -d "=" -f 2 | sed "s/'//g" | sed "s|http://||g")
#PRIVATE_GIT_READ=$(docker exec mbr_git_$network_number cat /massbit/massbitroute/app/src/sites/services/git/data/env/git.env  | grep GIT_PRIVATE_READ_URL  | cut -d "=" -f 2 | sed "s/'//g")

docker_compose_files=""
file="fisherman-docker-compose"

cat $TEMPLATE_PATH/${file}.template.yaml |  \
     sed "s|\[\[ENV_DIR\]\]|$ENV_DIR|g" | \
     sed "s|\[\[ROOT_DIR\]\]|$ROOT_DIR_MBR_TEST|g" | \
     sed "s|\[\[PROTOCOL\]\]|$PROTOCOL|g" | \
     sed "s/\[\[PROXY_TAG\]\]/$PROXY_TAG/g" | \
     sed "s/\[\[TEST_CLIENT_TAG\]\]/$TEST_CLIENT_TAG/g" | \
     sed "s/\[\[FISHERMAN_TAG\]\]/$FISHERMAN_TAG/g" | \
     #sed "s/\[\[RUN_ID\]\]/$network_number/g" | \
     sed "s/\[\[NETWORK_NUMBER\]\]/$network_number/g" | \
     sed "s/\[\[MASSBIT_CHAIN_TAG\]\]/$MASSBIT_CHAIN_TAG/g" | \
     sed "s/\[\[STAKING_TAG\]\]/$STAKING_TAG/g" | \
     sed "s/\[\[PORTAL_TAG\]\]/$PORTAL_TAG/g" | \
     sed "s/\[\[WEB_TAG\]\]/$WEB_TAG/g" | \
     sed "s/\[\[CHAIN_TAG\]\]/$CHAIN_TAG/g" | \
     sed "s/\[\[API_TAG\]\]/$API_TAG/g" | \
     sed "s/\[\[GIT_TAG\]\]/$GIT_TAG/g" | \
     sed "s/\[\[GWMAN_TAG\]\]/$GWMAN_TAG/g" | \
     sed "s/\[\[SESSION_TAG\]\]/$SESSION_TAG/g" | \
     sed "s/\[\[STAT_TAG\]\]/$STAT_TAG/g" | \
     sed "s/\[\[MONITOR_TAG\]\]/$MONITOR_TAG/g" | \
     #Ips
     sed "s/\[\[PROXY_IP\]\]/$PROXY_IP/g" | \
     sed "s/\[\[TEST_CLIENT_IP\]\]/$TEST_CLIENT_IP/g" | \
     sed "s/\[\[MASSBIT_CHAIN_IP\]\]/$MASSBIT_CHAIN_IP/g" | \
     sed "s/\[\[STAKING_IP\]\]/$STAKING_IP/g" | \
     sed "s/\[\[FISHERMAN_SCHEDULER_IP\]\]/$FISHERMAN_SCHEDULER_IP/g" | \
     sed "s/\[\[FISHERMAN_WORKER01_IP\]\]/$FISHERMAN_WORKER01_IP/g" | \
     sed "s/\[\[FISHERMAN_WORKER02_IP\]\]/$FISHERMAN_WORKER02_IP/g" | \
     sed "s/\[\[FISHER_ENVIRONMENT\]\]/$FISHER_ENVIRONMENT/g" | \
     sed "s/\[\[PORTAL_IP\]\]/$PORTAL_IP/g" | \
     sed "s/\[\[CHAIN_IP\]\]/$CHAIN_IP/g" | \
     sed "s/\[\[WEB_IP\]\]/$WEB_IP/g" | \
     sed "s/\[\[GWMAN_IP\]\]/$GWMAN_IP/g" | \
     sed "s/\[\[POSTGRES_IP\]\]/$POSTGRES_IP/g" | \
     sed "s/\[\[REDIS_IP\]\]/$REDIS_IP/g" | \
     sed "s/\[\[GIT_IP\]\]/$GIT_IP/g" | \
     sed "s/\[\[API_IP\]\]/$API_IP/g" | \
     sed "s/\[\[SESSION_IP\]\]/$SESSION_IP/g" | \
     sed "s/\[\[SCHEDULER_AUTHORIZATION\]\]/$SCHEDULER_AUTHORIZATION/g" | \
     sed "s|\[\[MASSBIT_ROUTE_SID\]\]|$MASSBIT_ROUTE_SID|g" | \
     sed "s|\[\[MASSBIT_ROUTE_PARTNER_ID\]\]|$MASSBIT_ROUTE_PARTNER_ID|g" \
    > $OUTPUT_PATH/${file}.template.tmp.yaml
cat $OUTPUT_PATH/${file}.template.tmp.yaml | sed "s|\[\[PRIVATE_GIT_READ\]\]|$PRIVATE_GIT_READ|g" > $OUTPUT_PATH/${file}.yaml




