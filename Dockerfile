################
##### Builder
FROM rust:1.61.0 as builder

WORKDIR /usr/src

# Create blank project
RUN USER=root cargo new fisherman-scheduler

# We want dependencies cached, so copy those first.
COPY scheduler/Cargo.toml  /usr/src/fisherman-scheduler/

# Set the working directory
WORKDIR /usr/src/fisherman-scheduler

## Install target platform (Cross-Compilation) --> Needed for Alpine
RUN rustup target add x86_64-unknown-linux-musl

COPY common  /usr/src/common
COPY wrap_wrk  /usr/src/wrap_wrk
COPY entity /usr/src/entity
COPY migration /usr/src/migration
COPY test_util /usr/src/test_util/

# Now copy in the rest of the sources
COPY scheduler /usr/src/fisherman-scheduler/

# This is the actual application build.
RUN cargo build  --release

# Now copy in the rest of the sources
COPY fisherman /usr/src/fisherman/
WORKDIR /usr/src/fisherman
# This is the actual application build.
RUN cargo build  --release

################
# ##### Runtime
FROM debian AS runtime 
WORKDIR /usr/local/bin

COPY scheduler/configs /usr/local/bin/configs
COPY scheduler/benchmark /usr/local/bin/benchmark
COPY script/supervisor.conf /etc/supervisor/conf.d/fisherman-scheduler.conf

RUN  supervisorctl reread && supervisorctl update

# Copy application binary from builder image
COPY --from=builder /usr/src/fisherman-scheduler/target/release/scheduler /usr/local/bin
COPY --from=builder /usr/src/fisherman/target/release/fisherman /usr/local/bin
RUN apt update && apt install supervisor -y
EXPOSE 80

# Run the application
CMD ["supervisord", "-n"]