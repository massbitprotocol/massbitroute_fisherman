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

# Now copy in the rest of the sources
COPY scheduler /usr/src/fisherman-scheduler/

RUN ls -la /usr/src/fisherman-scheduler/


# This is the actual application build.
RUN cargo build  --release

################
# ##### Runtime
FROM debian AS runtime 
WORKDIR /usr/local/bin

COPY scheduler/configs /usr/local/bin/configs
# Copy application binary from builder image
COPY --from=builder /usr/src/fisherman-scheduler/target/release/scheduler /usr/local/bin

EXPOSE 80

# Run the application
CMD ["/usr/local/bin/scheduler"]