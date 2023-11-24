# Use the Rust official image as the base image
FROM rust:1.73.0 as chef
WORKDIR /app

FROM chef AS builder
# Build application
COPY . .
RUN cargo build --release

# Start a new stage and copy the server binary from the builder stage
FROM debian:bookworm-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/shard /usr/local/bin/

# Check if KEY environment variable is set and create ~/.shard/key file
RUN if [ -n "$KEY" ]; then \
    mkdir -p ~/.shard && echo "$KEY" > ~/.shard/key; \
  fi

# Check if BOOTSTRAPPER environment variable is set and create ~/.shard/conf.toml file
RUN if [ -n "$BOOTSTRAPPER" ]; then \
    mkdir -p ~/.shard && echo "bootstrappers = [ \"$BOOTSTRAPPER\" ]" > ~/.shard/conf.toml; \
  fi
  
# Check if LISTEN_ADDR environment variable is set, and conditionally set CMD
CMD [ "sh", "-c", "if [ -n \"$LISTEN_ADDR\" ]; then exec shard provide --listen-address=$LISTEN_ADDR; else exec shard provide; fi" ]
