# Use the Rust official image as the base image
FROM rust:1.73.0 as chef
RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --package mpcnet --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --package mpcnet

# Start a new stage and copy the server binary from the builder stage
FROM debian:bookworm-slim AS runtime
WORKDIR /app
COPY --from=builder /app/target/release/mpcnet /usr/local/bin/

# Command to run the application
CMD ["mpcnet"]
