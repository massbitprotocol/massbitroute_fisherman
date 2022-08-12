#!/bin/bash
if [ -z "$1" ]
then
    exit 1
fi
NETWORK_NUMBER=$1

docker exec mbr_api_$NETWORK_NUMBER bash -c '/massbit/massbitroute/app/src/sites/services/api/cmd_server start nginx'
docker exec mbr_portal_api_$NETWORK_NUMBER bash -c 'cd /app; npm run dbm:init'
docker exec mbr_db_$NETWORK_NUMBER bash -c 'bash /docker-entrypoint-initdb.d/3_init_user.sh'
