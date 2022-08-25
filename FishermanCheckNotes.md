# List of Fisherman checks
(Replace $PROVIDER_IP by provider ip)
## Round trip time
Check if the node is reachable and get response time
```bash
curl --location --request GET 'https://$PROVIDER_IP/_rtt' \
--header 'Content-Type: application/json'
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

