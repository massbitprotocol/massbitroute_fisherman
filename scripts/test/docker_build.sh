#!/bin/bash

cd ../..
cargo build
docker build -t massbit/massbitroute_fisherman:v0.1.1-web3-grant .