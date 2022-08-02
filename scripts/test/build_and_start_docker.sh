#!/bin/bash

cd ../../
docker build -t mbr_fisherman:v0.1-dev .

cd scripts/test
docker-compose up -d