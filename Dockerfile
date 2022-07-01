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

# Now copy in the rest of the sources
COPY scheduler /usr/src/fisherman-scheduler/

RUN ls -la /usr/src/fisherman-scheduler/


# This is the actual application build.
RUN cargo build  --release

################
# ##### Runtime
FROM ubuntu AS runtime 

# Copy application binary from builder image
COPY --from=builder /usr/src/fisherman-scheduler/target/release/scheduler /usr/local/bin

EXPOSE 80

ENV RUST_LOG=info
ENV RUST_LOG_TYPE=console
ENV DATABASE_URL=postgres://postgres:postgres@db:5432/massbit-fisherman
ENV DOMAIN=massbitroute.dev
ENV PORTAL_AUTHORIZATION=g2xnS1uKr4Ko7tPApdceP4NSOKhhbWbX
ENV URL_GATEWAYS_LIST=https://portal.massbitroute.dev/mbr/gateway/list/verify
ENV URL_NODES_LIST=https://portal.massbitroute.dev/mbr/node/list/verify
ENV SCHEDULER_ENDPOINT=0.0.0.0:80
# Run the application
CMD ["/usr/local/bin/scheduler"]