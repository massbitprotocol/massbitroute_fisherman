# List of Fisherman checks
(Replace $PROVIDER_IP by provider ip)
## Round trip time
- Check if the node is reachable and get response time
```bash
curl --location --request GET 'https://$PROVIDER_IP/_rtt' \
--header 'Content-Type: application/json'
```
- Verify parameter `dockerize/scheduler_config/configs.production/tasks/http_request/round_trip_time.json`:
```bash
"histogram_percentile": 95,
"response_duration": 500,
"success_percent": 50,
"number_for_decide": 5
```
## Latest Block
Check if the node can return the latest block data and whether it is too faraway in comparison with the latest block in system.
### Ethereum node
```bash
curl --location --request POST 'https://$PROVIDER_IP' \
--header 'x-api-key: $PROVIDER_API' \
--header 'Content-Type: application/json' \
--header 'Host: $PROVIDER_ID.node.mbr.massbitroute.net' \
--data-raw '{
    "jsonrpc": "2.0",
    "method": "eth_getBlockByNumber",
    "params": [
        "latest",
        false
    ],
    "id": 1
}'
```
- Verify parameter `dockerize/scheduler_config/configs.production/tasks/http_request/eth_latest_block.json`:
```bash
"request_timeout": 5000
"late_duration":  1200
```
### Ethereum gateway
```bash
curl --location --request POST 'https://$PROVIDER_IP' \
--header 'x-api-key: $PROVIDER_API' \
--header 'Content-Type: application/json' \
--header 'Host: $PROVIDER_ID.gw.mbr.massbitroute.net' \
--data-raw '{
    "jsonrpc": "2.0",
    "method": "eth_getBlockByNumber",
    "params": [
        "latest",
        false
    ],
    "id": 1
}'
```
### Polkadot node
```bash
curl --location --request POST 'https://$PROVIDER_IP' \
--header 'x-api-key: $PROVIDER_API' \
--header 'Content-Type: application/json' \
--header 'Host: $PROVIDER_ID.node.mbr.massbitroute.net' \
--data-raw '{
    "jsonrpc": "2.0",
    "method": "chain_getBlock",
    "params": [],
    "id": 1
}'
```
- Verify parameter `dockerize/scheduler_config/configs.production/tasks/http_request/dot_latest_block.json`:
```bash
"request_timeout": 5000
"max_block_missing": 200
```
### Polkadot gateway
```bash
curl --location --request POST 'https://$PROVIDER_IP' \
--header 'x-api-key: $PROVIDER_API' \
--header 'Content-Type: application/json' \
--header 'Host: $PROVIDER_ID.gw.mbr.massbitroute.net' \
--data-raw '{
    "jsonrpc": "2.0",
    "method": "chain_getBlock",
    "params": [],
    "id": 1
}'
```

## Benchmark
- Check if the node/gateway can respond correctly and quickly
- Verify parameter `dockerize/scheduler_config/configs.production/tasks/benchmark/default.json`:

```bash
"benchmark_duration": 15000
"benchmark_rate": 10
"response_threshold": 500
"judge_histogram_percentile": 95
"response_threshold": 500
```

## Benchmark
- Check if the node/gateway can respond websocket connection correctly.