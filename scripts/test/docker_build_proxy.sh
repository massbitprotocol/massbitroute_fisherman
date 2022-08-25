#!/bin/bash
ROOT=$(realpath $(dirname $(realpath $0))/)
declare -A hosts
cd $ROOT/docker-proxy
#git clone http://massbit:DaTR__SGr89IjgvcwBtJyg0v_DFySDwI@git.massbitroute.net/massbitroute/ssl.git -b shamu ssl
_IFS=$IFS
while IFS=":" read -r server_name ip
do
  hosts[$server_name]=$ip
done < <(cat $ROOT/docker-proxy/hosts)
IFS=$_IFS
cat common.conf > proxy.conf
domain=massbitroute.net
servers=("api" "portal" "admin-api" "dapi" "staking" "hostmaster" "ns1" "ns2")
for server in ${servers[@]}; do
  server_name=$server.$domain
  echo "Generate server block for $server_name with ip ${hosts[$server_name]}"
  #openssl req -x509 -nodes -days 7300 -newkey rsa:2048 -subj "/C=PE/ST=Lima/L=Lima/O=Acme Inc. /OU=IT Department/CN=$server_name" -keyout $ROOT/docker-proxy/ssl/selfsigned/${server_name}.key -out $ROOT/docker-proxy/ssl/selfsigned/${server_name}.cert
  cat server.template | sed "s/\[\[SERVER_NAME\]\]/$server_name/g" | sed "s/\[\[DOMAIN\]\]/$domain/g" |  sed "s/\[\[IP\]\]/${hosts[$server_name]}/g" >> proxy.conf
done

#domain=mbr.massbitroute.net
#servers=("stat" "monitor")
#for server in ${servers[@]}; do
#  server_name=$server.$domain
#  echo "Generate server block for $server_name with ip ${hosts[$server_name]}"
#  #openssl req -x509 -nodes -days 7300 -newkey rsa:2048 -subj "/C=PE/ST=Lima/L=Lima/O=Acme Inc. /OU=IT Department/CN=$server_name" -keyout $ROOT/docker-proxy/ssl/selfsigned/${server_name}.key -out $ROOT/docker-proxy/ssl/selfsigned/${server_name}.cert
#  cat server.template | sed "s/\[\[SERVER_NAME\]\]/$server_name/g" | sed "s/\[\[DOMAIN\]\]/$domain/g" |  sed "s/\[\[IP\]\]/${hosts[$server_name]}/g" >> proxy.conf
#done

#domain=monitor.mbr.massbitroute.net
#servers=("gateway-dot-mainnet" "gateway-eth-mainnet" "node-eth-mainnet" "node-dot-mainnet")
#for server in ${servers[@]}; do
#  server_name=$server.$domain
#  echo "Generate server block for $server_name with ip ${hosts[$server_name]}"
#  #openssl req -x509 -nodes -days 7300 -newkey rsa:2048 -subj "/C=PE/ST=Lima/L=Lima/O=Acme Inc. /OU=IT Department/CN=$server_name" -keyout $ROOT/docker-proxy/ssl/selfsigned/${server_name}.key -out $ROOT/docker-proxy/ssl/selfsigned/${server_name}.cert
#  cat server.template | sed "s/\[\[SERVER_NAME\]\]/$server_name/g" | sed "s/\[\[DOMAIN\]\]/$domain/g" |  sed "s/\[\[IP\]\]/${hosts[$server_name]}/g" >> proxy.conf
#done

#domain=stat.mbr.massbitroute.net
#servers=("gateway-dot-mainnet" "gateway-eth-mainnet" "node-eth-mainnet" "node-dot-mainnet")
#for server in ${servers[@]}; do
#  server_name=$server.$domain
#  echo "Generate server block for $server_name with ip ${hosts[$server_name]}"
#  #openssl req -x509 -nodes -days 7300 -newkey rsa:2048 -subj "/C=PE/ST=Lima/L=Lima/O=Acme Inc. /OU=IT Department/CN=$server_name" -keyout $ROOT/docker-proxy/ssl/selfsigned/${server_name}.key -out $ROOT/docker-proxy/ssl/selfsigned/${server_name}.cert
#  cat server.template | sed "s/\[\[SERVER_NAME\]\]/$server_name/g" | sed "s/\[\[DOMAIN\]\]/$domain/g" |  sed "s/\[\[IP\]\]/${hosts[$server_name]}/g" >> proxy.conf
#done


domain=fisherman.massbitroute.net
server_name=scheduler.$domain
echo "Generate server block for $server_name with ip ${hosts[$server_name]}"
#openssl req -x509 -nodes -days 7300 -newkey rsa:2048 -subj "/C=PE/ST=Lima/L=Lima/O=Acme Inc. /OU=IT Department/CN=$server_name" -keyout $ROOT/docker-proxy/ssl/selfsigned/${server_name}.key -out $ROOT/docker-proxy/ssl/selfsigned/${server_name}.cert
cat server.template | sed "s/\[\[SERVER_NAME\]\]/$server_name/g" | sed "s/\[\[DOMAIN\]\]/$domain/g" |  sed "s/\[\[IP\]\]/${hosts[$server_name]}/g" >> proxy.conf

#workers=("worker01" "worker02")
#for worker in ${workers[@]}; do
#  server_name=$worker.$domain
#  echo "Generate server block for $server_name with ip ${hosts[$server_name]}"
  #openssl req -x509 -nodes -days 7300 -newkey rsa:2048 -subj "/C=PE/ST=Lima/L=Lima/O=Acme Inc. /OU=IT Department/CN=$server_name" -keyout $ROOT/docker-proxy/ssl/selfsigned/${server_name}.key -out $ROOT/docker-proxy/ssl/selfsigned/${server_name}.cert
#  cat server.template \
#    | sed "s/\[\[SERVER_NAME\]\]/$server_name/g" \
#    | sed "s/\[\[DOMAIN\]\]/$domain/g" \
#    | sed "s/\[\[IP\]\]/${hosts[$server_name]}:4040/g" >> proxy.conf
#done




#mkdir ssl/live/api.ipapi.com
#server_name=api.ipapi.com
#echo "Generate server block for $server_name with ip ${hosts[$server_name]}"
#openssl req -x509 -nodes -days 7300 -newkey rsa:2048 -subj "/C=PE/ST=Lima/L=Lima/O=Acme Inc. /OU=IT Department/CN=$server_name" -keyout $ROOT/docker-proxy/ssl/live/${server_name}/privkey.pem -out $ROOT/docker-proxy/ssl/live/${server_name}/cert.pem
#cat server.template | sed "s/\[\[SERVER_NAME\]\]/$server_name/g" | sed "s/\[\[DOMAIN\]\]/$server_name/g" |  sed "s/\[\[IP\]\]/${hosts[$server_name]}/g" >> proxy.conf

docker build -t massbit/massbitroute_test_proxy:v0.1.0-rel .
