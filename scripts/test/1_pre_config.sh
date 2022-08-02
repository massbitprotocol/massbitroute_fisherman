#!/bin/bash

docker exec mbr_api bash -c '/massbit/massbitroute/app/src/sites/services/api/cmd_server start nginx'
docker exec mbr_portal_api bash -c 'cd /app; npm run dbm:init'
docker exec mbr_db bash -c 'bash /docker-entrypoint-initdb.d/init_user.sh'
