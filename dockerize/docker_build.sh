#docker build -f RustBuilderDockerfile -t rustbuilder:1.61.0 .
#docker run -it --name rustbuilder -v $(pwd)/..:/fisherman rustbuilder:1.61.0 bash -c 'cd /fisherman && cargo build --release'
docker-compose -f docker-compose.yml up -d
docker exec -it rustbuilder bash -c 'cd /fisherman && cargo build --release'
DEFAULT_TAG=v0.1.1-web3-grant
cp -r ../scripts/benchmark .
cp ../target/release/scheduler .
cp ../target/release/fisherman .
cp ../target/release/mbr_stats .
cp ../scripts/build_docker/services .
docker build -f RuntimeDockerfile -t massbit/massbitroute_fisherman:${1:-$DEFAULT_TAG} .
rm -rf ./benchmark ./scheduler ./fisherman ./services ./mbr_stats
