#!/bin/bash

# run only once time on new machine
docker network create -d bridge --gateway 172.24.24.1 --subnet 172.24.24.0/24   mbr_test_network