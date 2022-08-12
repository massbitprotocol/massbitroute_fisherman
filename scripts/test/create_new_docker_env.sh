#!/bin/bash


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
network_number="24"

cat docker-compose.yaml.template |  \
	 sed "s/\[\[RUN_ID\]\]/$network_number/g" | \
	 sed "s/\[\[NETWORK_NUMBER\]\]/$network_number/g" \
    > docker-compose.yaml

    