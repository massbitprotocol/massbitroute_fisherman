#docker build -f RustBuilderDockerfile -t rustbuilder:1.61.0 .
docker run -it -name rustbuilder -v $(pwd)/..:/fisherman rustbuilder:1.61.0 bash -c 'cd /fisherman && cargo build --release'
docker exec -it rustbuilder bash -c 'cd /fisherman && cargo build --release'

cp -r ../scripts/benchmark .
cp ../target/release/scheduler .
cp ../target/release/fisherman .
cp ../scripts/build_docker/services .
docker build -f RuntimeDockerfile -t massbit/massbitroute_fisherman:${1-v0.1.0-dev} .
rm -rf ./benchmark ./scheduler ./fisherman ./services
