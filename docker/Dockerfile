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
COPY --from=builder /app/target/release/mpcnet /usr/local/bin/

# Command to run the application
CMD ["mpcnet"]
