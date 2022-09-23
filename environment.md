# Scheduler module
## Environment variables
### Public
Example
```bash
COMMON_CONFIG_FILE=~/massbitroute_fisherman/common/configs/common.json
ENVIRONMENT=local                             #Deploy env: local/docker_test/release/production
IS_REGULAR_REPORT=false                       #Enable regular report
IS_VERIFY_REPORT=false                        #Enable verify report
PATH_GATEWAYS_LIST=mbr/gateway/list/verify    #Portal get gateway path
PATH_NODES_LIST=mbr/node/list/verify          #Portal get node path
PATH_PORTAL_PROVIDER_REPORT=mbr/benchmark     #Portal report regular path
PATH_PORTAL_PROVIDER_VERIFY=mbr/verify        #Portal verify regular path
REPORT_CALLBACK=http://127.0.0.1:3031/report  #Schedule endpoint for worker callback
RUST_LOG=info                                 #Log lv
SCHEME=https                                  #Url: http/https 
URL_CHAIN=wss://chain-beta.massbitroute.net    
URL_PORTAL=https://portal.massbitroute.net
```

### Secret
Example:
```bash
PORTAL_AUTHORIZATION="g2xnS1uKr4Ko7tPApdceP4NSOKxxxxx"
DATABASE_URL=postgres://massbit:xxxxx@localhost:5432/massbit-fisherman                        
SCHEDULER_AUTHORIZATION=11967d5e9addc5416ea9224eeexxxxxx                                      #Authorize header for worker submit result
SIGNER_PHRASE="xxxxx xxxxx obey lake curtain smoke basket hold race lonely fit walk//xxxxxx"  #Signer for interact with Massbit chain
```
## Config files
- `scheduler/configs/scheduler.json`:  Parameter for scheduler 
```json
{
  "checking_component_status": "staked",
  "check_ping_pong_interval": 2,
  "check_logic_interval": 2,
  "check_benchmark_interval": 3,
  "update_provider_list_interval": 30,
  "regular_plan_generate_interval": 10,
  "generate_new_regular_timeout": 90,
  "plan_expiry_time": 600,
  "update_worker_list_interval": 30
}
```

- `scheduler/configs/task/task_master`:  Enable tasks for config Regular and Verification 
```json
{
  "regular": ["HttpRequest"],
  "verification": ["HttpRequest", "Benchmark", "Websocket"]
}
```
- `scheduler/configs/task/benchmark`:  Directory for config benchmark task
```
default.json            #Default config
verify_dot_node.json    #Verify dot node config
verify_eth_node.json    #Verify eth node config
verify_gateway.json     #Verify gateway node config
```

- `scheduler/configs/task/http_request`:  Directory for config http request task, include LatestBlock and RoundTripTime task.
```
default.json            #Default config
dot_latest_block.json   #Task check latest block dot provider config
eth_latest_block.json   #Task check latest block eth provider config
round_trip_time.json    #Task check round trip time config
```

- `scheduler/configs/task/websocket`:  Directory for config websocket
```
default.json            #Default config
dot_latest_block.json   #Task check websocket dot provider config
eth_latest_block.json   #Task check websocket eth provider config
```
# Fisherman worker module
## Environment variables
### Public
Example
```bash
export DOMAIN=massbitroute.net
export RUST_LOG=debug
export SCHEDULER_ENDPOINT=https://scheduler.fisherman.$DOMAIN
export WORKER_IP=$(curl ifconfig.me)
export ZONE={{ZONE}}
export WORKER_ID="$ZONE-$WORKER_IP"
export WORKER_ENDPOINT=http://$WORKER_IP:4040
export WORKER_SERVICE_ENDPOINT=0.0.0.0:4040
export BENCHMARK_WRK_PATH=/opt/fisherman/benchmark      #Relative path to benchmark bin + config file
export COMMON_CONFIG_FILE=/opt/fisherman/common.json
export ENVIRONMENT=local                                #Deploy env: local/docker_test/release/production
export SCHEME=https                                     #Url: http/https 
``` 

### Secret
Example:
```bash                     
SCHEDULER_AUTHORIZATION=11967d5e9addc5416ea9224eeexxxxxx                                      #Authorize header for worker submit result
```
# Stats module
## Run parameter
```bash
run --manifest-path ~/massbitroute_fisherman/mbr_stats/Cargo.toml  -- \
update-stats --prometheus-url \
stat.mbr.massbitroute.net/__internal_prometheus  \
--list-project-url https://portal.massbitroute.net/mbr/d-apis/project/list/verify 
```

## Environment variables
### Public
Example
```bash
SCHEME=https
URL_CHAIN=wss://chain.massbitroute.net
ENVIRONMENT=local
```
### Secret
Example:
```bash                     
SCHEDULER_AUTHORIZATION=11967d5e9addc5416ea9224eeexxxxxx                                      #Authorize header for worker submit result
```

