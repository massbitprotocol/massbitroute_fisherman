#!/bin/bash

#./wrk --latency -t20 -c20 -d15s -R10 -s massbit_old.lua https://34.101.146.31 -- \
#lSP1lFN9I_izEzRi_jBapA 058a6e94-8b65-46ad-ab52-240a7cb2c36a.node.mbr.massbitroute.net \
#node / eth

./wrk --latency -t20 -c20 -d15s -R10 -s massbit.lua http://34.101.146.31 \
-H "Content-Type: application/json" -H "X-Api-Key: lSP1lFN9I_izEzRi_jBapA" \
-H "Host: 058a6e94-8b65-46ad-ab52-240a7cb2c36a.node.mbr.massbitroute.net" \
-H "Connection: Close" -- \
--method POST --body '{"id": "blockNumber", "jsonrpc": "2.0", "method": "eth_getBlockByNumber", "params": ["latest", false]}'
