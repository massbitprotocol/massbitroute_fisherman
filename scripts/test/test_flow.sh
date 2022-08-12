#!/bin/bash
TEST_USERNAME=demo
TEST_PASSWORD=Codelight123
blockchain=eth
dataSource="http:\/\/34.81.232.186:8545"
dataSourceWs="ws:\/\/34.81.232.186:8546"
nodePrefix="$(echo $RANDOM | md5sum | head -c 5)"
MEMONIC="bottom drive obey lake curtain smoke basket hold race lonely fit walk//Alice"

docker-compose down
(
  cd docker-node || exit
  docker-compose down
)
(
  cd docker-gateway || exit
  docker-compose down
)
cd ../


#-------------------------------------------
# Docker build
#-------------------------------------------
bash docker_build.sh
#bash docker_build_proxy.sh
#-------------------------------------------
# Docker up
#-------------------------------------------
docker-compose up -d
truncate -s 0 ./docker-proxy/logs/*.*
sleep 10
bash 1_pre_config.sh || exit 1
if 
docker exec mbr_db bash -c 'bash /docker-entrypoint-initdb.d/2_clean_node.sh'

#-------------------------------------------
# Log into Portal
#-------------------------------------------
bearer=
while [[ "x$bearer" == "x" ]] || [[ "$bearer" == "null" ]]; do
  echo "Try login..."
  bearer=$(curl -k --location --request POST 'https://portal.massbitroute.net/auth/login' --header 'Content-Type: application/json' \
          --data-raw "{\"username\": \"$TEST_USERNAME\", \"password\": \"$TEST_PASSWORD\"}"| jq  -r ". | .accessToken")
  sleep 5
done

userID=$(curl -k 'https://portal.massbitroute.net/user/info' \
  -H 'Accept: application/json, text/plain, */*' \
  -H 'Accept-Language: en-US,en;q=0.9' \
  -H "Authorization: Bearer $bearer" | jq -r ". | .id")
#-------------------------------------------
# create  node/gw in Portal
#-------------------------------------------
now=$(date)
echo "Create new node and gw in Portal: In Progress at $now"
curl -k --location --request POST 'https://portal.massbitroute.net/mbr/node' \
  --header "Authorization: Bearer  $bearer" \
  --header 'Content-Type: application/json' \
  --data-raw "{
      \"name\": \"mb-dev-node-$nodePrefix\",
      \"blockchain\": \"$blockchain\",
      \"zone\": \"AS\",
      \"dataSource\": \"$dataSource\",
      \"network\": \"mainnet\",
      \"dataSourceWs\":\"$dataSourceWs\"
  }" | jq -r '. | .id, .appKey' | sed -z -z 's/\n/,/g;s/,$/,AS\n/' >nodelist.csv

curl -k --location --request POST 'https://portal.massbitroute.net/mbr/gateway' \
  --header "Authorization: Bearer  $bearer" \
  --header 'Content-Type: application/json' \
  --data-raw "{
    \"name\":\"MB-dev-gateway-$nodePrefix\",
    \"blockchain\":\"$blockchain\",
    \"zone\":\"AS\",
    \"network\":\"mainnet\"}" | jq -r '. | .id, .appKey' | sed -z -z 's/\n/,/g;s/,$/,AS\n/' >gatewaylist.csv

#-------------------------------------------
# check if node/gw are created in Portal successfully
#-------------------------------------------
GATEWAY_ID=$(cut -d ',' -f 1 gatewaylist.csv)
NODE_ID=$(cut -d ',' -f 1 nodelist.csv)

echo "        NODE/GW INFO        "
echo "----------------------------"
echo "Gateway ID: $GATEWAY_ID"
echo "Node ID: $NODE_ID"
echo "----------------------------"

#-------------------------------------------
# Update docker-compose for node
#-------------------------------------------
GATEWAY_APP_KEY=$(cut -d ',' -f 2  gatewaylist.csv)
NODE_APP_KEY=$(cut -d ',' -f 2 nodelist.csv)


cat docker-node/docker-compose.yaml.template | sed "s/\[\[NODE_ID\]\]/$NODE_ID/g" | \
	 sed "s/\[\[BLOCKCHAIN\]\]/$blockchain/g" | \
         sed "s/\[\[DATA_URL\]\]/$dataSource/g" | \
	 sed "s/\[\[APP_KEY\]\]/$NODE_APP_KEY/g" | \
	 sed "s/\[\[USER_ID\]\]/$userID/g" > docker-node/docker-compose.yaml
cat docker-gateway/docker-compose.yaml.template | sed "s/\[\[GATEWAY_ID\]\]/$GATEWAY_ID/g" | \
         sed "s/\[\[BLOCKCHAIN\]\]/$blockchain/g" | \
         sed "s/\[\[DATA_URL\]\]/$dataSource/g" | \
         sed "s/\[\[APP_KEY\]\]/$GATEWAY_APP_KEY/g" | \
         sed "s/\[\[USER_ID\]\]/$userID/g" > docker-gateway/docker-compose.yaml

#-------------------------------------------
# Create docker node
#-------------------------------------------
cd docker-node
docker-compose up -d

#-------------------------------------------
# Check if nodes are verified
#-------------------------------------------
while [[ "$node_status" != "approved" ]]; do
  echo "Checking node status: In Progress"

  node_status=$(curl -k --location --request GET "https://portal.massbitroute.net/mbr/node/$NODE_ID" \
    --header "Authorization: Bearer $bearer" | jq -r ". | .status")

  echo "---------------------------------"
  echo "Node status: $node_status"
  echo "---------------------------------"
  sleep 10
done
now=$(date)
echo "Checking node approved status: Passed at $now"

#-------------------------------------------
# Test staking for NODES
#-------------------------------------------

node_staking_response=$(curl --location --request POST 'http://staking.massbitroute.net/massbit/staking-provider' \
  --header 'Content-Type: application/json' --data-raw "{
    \"memonic\": \"$MEMONIC\",
    \"providerId\": \"$NODE_ID\",
    \"providerType\": \"Node\",
    \"blockchain\": \"$blockchain\",
    \"network\": \"mainnet\",
    \"amount\": \"100\"
}" | jq -r ". | .status")
if [[ "$node_staking_response" != "success" ]]; then
  echo "Node staking: Failed"
  exit 1
fi
while [[ "$node_status" != "staked" ]]; do
  echo "Checking node status: In Progress"

  node_status=$(curl -k --location --request GET "https://portal.massbitroute.net/mbr/node/$NODE_ID" \
    --header "Authorization: Bearer $bearer" | jq -r ". | .status")
  now=$(date)
  echo "---------------------------------"
  echo "Node status at:$now is $node_status"
  echo "---------------------------------"
  sleep 10
done
now=$(date)
echo "Node staked: Passed at $now"

##-------------------------------------------
## Create docker gateway
##-------------------------------------------

cd ../docker-gateway
docker-compose up -d

##-------------------------------------------
## Check if Gateway are verified
##-------------------------------------------
while [[ "$gateway_status" != "approved" ]]; do
  echo "Checking Gateway status: In Progress"

  gateway_status=$(curl -k --location --request GET "https://portal.massbitroute.net/mbr/gateway/$GATEWAY_ID" \
    --header "Authorization: Bearer $bearer" | jq -r ". | .status")


  echo "---------------------------------"
  echo "Gateway status: $gateway_status"
  echo "---------------------------------"
  sleep 10
done
now=$(date)
echo "Checking node verified status: Passed at $now"



#-------------------------------------------
# Test staking for GW
#-------------------------------------------
# stake gateway
now=$(date)
echo "Wait a minute for staking node..."
echo "$now"
gateway_staking_response=$(curl --location --request POST 'http://staking.massbitroute.net/massbit/staking-provider' \
  --header 'Content-Type: application/json' --data-raw "{
    \"memonic\": \"$MEMONIC\",
    \"providerId\": \"$GATEWAY_ID\",
    \"providerType\": \"Gateway\",
    \"blockchain\": \"$blockchain\",
    \"network\": \"mainnet\",
    \"amount\": \"100\"
}" | jq -r ". | .status")


if [[ "$gateway_staking_response" != "success" ]]; then
  echo "Gateway staking status: Failed "
  exit 1
fi
while [[ "$gateway_status" != "staked" ]]; do
  echo "Checking Gateway status: In Progress"

  gateway_status=$(curl -k --location --request GET "https://portal.massbitroute.net/mbr/gateway/$GATEWAY_ID" \
    --header "Authorization: Bearer $bearer" | jq -r ". | .status")

  now=$(date)
  echo "---------------------------------"
  echo "Gateway status at $now is $gateway_status"
  echo "---------------------------------"
  sleep 10
done
now=$(date)
echo "Gateway staked: Passed at $now"

#-------------------------------------------
# Turn off NODES/GW
#-------------------------------------------
echo "Turning off gateway"
docker-compose down
cd ../docker-node
echo "Turning off node"
docker-compose down

while [[ "$node_status" != "investigate" ]]; do
  echo "Checking node status: In Progress"

  node_status=$(curl -k --location --request GET "https://portal.massbitroute.net/mbr/node/$NODE_ID" \
    --header "Authorization: Bearer $bearer" | jq -r ". | .status")
  now=$(date)
  echo "---------------------------------"
  echo "Node status at $now is $node_status"
  echo "---------------------------------"
  sleep 10
done
now=$(date)
echo "Checking node reported status: investigate at $now"


##-------------------------------------------
## Check if gateways are verified
##-------------------------------------------
while [[ "$gateway_status" != "investigate" ]]; do
  echo "Checking gateway status: In Progress"

  gateway_status=$(curl -k --location --request GET "https://portal.massbitroute.net/mbr/gateway/$GATEWAY_ID" \
    --header "Authorization: Bearer $bearer" | jq -r ". | .status")

  now=$(date)
  echo "---------------------------------"
  echo "Gateway status at $now is $gateway_status"
  echo "---------------------------------"
  sleep 10
done
now=$(date)
echo "Checking gateway verified status: investigate at $now"

docker-compose down
