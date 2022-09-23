################
##### Builder
FROM rust:1.61.0 as builder

WORKDIR /usr/src

## Install target platform (Cross-Compilation) --> Needed for Alpine
RUN rustup target add x86_64-unknown-linux-musl
RUN apt-get update -q
RUN apt-get upgrade -y
RUN apt-get install -y libprotobuf-c-dev protobuf-compiler

COPY . /usr/src/
# This is the actual application build.

RUN cargo build  --release


################
# ##### Runtime
FROM debian AS runtime
WORKDIR /usr/local/bin

COPY scripts/benchmark /usr/local/bin/benchmark
#COPY scheduler/configs /usr/local/bin/configs
#COPY scripts/build_docker/supervisor.conf /etc/supervisor/conf.d/fisherman-scheduler.conf
#COPY scripts/build_docker/run_scheduler_docker.sh scripts/build_docker/run_worker_docker.sh /usr/local/bin/

# Copy application binary from builder image
COPY --from=builder /usr/src/target/release/scheduler /usr/local/bin
COPY --from=builder /usr/src/target/release/fisherman /usr/local/bin
COPY --from=builder /usr/src/target/release/mbr_stats /usr/local/bin
COPY scripts/build_docker/services /etc/services
RUN apt update && apt install supervisor -y
EXPOSE 80

# Run the application
CMD ["supervisord", "-n"]
