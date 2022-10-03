#!/bin/bash
export BUILDER_IMAGE=fishermanbuilder:1.61.0
export BUILDER_CONTAINER=fishermanbuilder
export DEFAULT_TAG=v0.1.1
export FISHER_ENVIRONMENT=docker_test
#docker run -it --name rustbuilder -v $(pwd)/..:/fisherman rustbuilder:1.61.0 bash -c 'cd /fisherman && cargo build --release'
CHECKBUILDER=$(docker image inspect $BUILDER_IMAGE >/dev/null 2>&1 && echo 1 || echo 0)
if [ "$CHECKBUILDER" == "0" ]; then
  docker build -f RustBuilderDockerfile -t $BUILDER_IMAGE .
fi
docker-compose -f builder-docker-compose.yml up -d
docker exec -it $BUILDER_CONTAINER bash -c "cd /fisherman && cargo build --release"

cp -r ../scripts/benchmark .
cp ../target/release/scheduler .
cp ../target/release/fisherman .
cp ../target/release/mbr_stats .
cp ../scripts/build_docker/services .
docker build --build-arg FISHER_ENVIRONMENT=$FISHER_ENVIRONMENT -f RuntimeDockerfile -t massbit/massbitroute_fisherman:${1:-$DEFAULT_TAG} .
rm -rf ./benchmark ./scheduler ./fisherman ./services ./mbr_stats
