# DEPLOY NOTES
## Scheduler module
### Environment variables
#### Public
- `COMMON_CONFIG_FILE`: path to common.json 
- `ENVIRONMENT`: running environment
- `COMMON_CONFIG_FILE`: path to common.json
- `COMMON_CONFIG_FILE`: path to common.json
- `COMMON_CONFIG_FILE`: path to common.json
- `COMMON_CONFIG_FILE`: path to common.json
- `COMMON_CONFIG_FILE`: path to common.json
- `COMMON_CONFIG_FILE`: path to common.json
- `COMMON_CONFIG_FILE`: path to common.json
- `COMMON_CONFIG_FILE`: path to common.json
- `COMMON_CONFIG_FILE`: path to common.json

- ENVIRONMENT=local
IS_REGULAR_REPORT=false
IS_VERIFY_REPORT=false
PATH_GATEWAYS_LIST=mbr/gateway/list/verify
PATH_NODES_LIST=mbr/node/list/verify
PATH_PORTAL_PROVIDER_REPORT=mbr/benchmark
PATH_PORTAL_PROVIDER_VERIFY=mbr/verify
REPORT_CALLBACK=http://127.0.0.1:3031/report
RUST_LOG=info
SCHEME=https
URL_CHAIN=wss://chain-beta.massbitroute.net
URL_PORTAL=https://portal.massbitroute.net
Example
```
COMMON_CONFIG_FILE=/home/user/work/block_chain/project-massbit-route/massbitroute_fisherman/common/configs/common.json
ENVIRONMENT=local
IS_REGULAR_REPORT=false
IS_VERIFY_REPORT=false
PATH_GATEWAYS_LIST=mbr/gateway/list/verify
PATH_NODES_LIST=mbr/node/list/verify
PATH_PORTAL_PROVIDER_REPORT=mbr/benchmark
PATH_PORTAL_PROVIDER_VERIFY=mbr/verify
REPORT_CALLBACK=http://127.0.0.1:3031/report
RUST_LOG=info
SCHEME=https
URL_CHAIN=wss://chain-beta.massbitroute.net
URL_PORTAL=https://portal.massbitroute.net
```
#### Secret
Example:
```
PORTAL_AUTHORIZATION="g2xnS1uKr4Ko7tPApdceP4NSOKxxxxx"
DATABASE_URL=postgres://massbit:xxxxx@localhost:5432/massbit-fisherman
SCHEDULER_AUTHORIZATION=11967d5e9addc5416ea9224eeexxxxxx
SIGNER_PHRASE="xxxxx xxxxx obey lake curtain smoke basket hold race lonely fit walk//xxxxxx"
```
