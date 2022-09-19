# Test
```bash
cargo test
```

# Build docker
```bash
cd dockerize
# First time ONLY
docker build -f RustBuilderDockerfile -t rustbuilder:1.61.0 .
# Build with tag 
bash docker_build.sh <TAG>
```

# Deploy
## Scheduler module
```bash
cd script/deploy
bash deploy_scheduler.sh
```
## Fisherman worker module
## Scheduler
```bash
cd script/deploy
bash deploy_worker.sh
```
