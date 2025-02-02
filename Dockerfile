FROM rust:1 AS chef
# We only pay the installation cost once,
# it will be cached from the second build onwards
RUN apt-get update -y && \
    cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare  --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin web-server

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime

# Install OpenSSL and other necessary libraries
RUN apt-get update && \
    apt-get install -y libssl3 ca-certificates && \
    apt-get clean && \
    rm -rf /var/lib/apt/lists/* \

ENV LISTEN_PORT=3000
ENV ELASTICSEARCH_ADDRESS=http://elasticsearch:9200

WORKDIR /app
COPY --from=builder /app/target/release/web-server /usr/local/bin
ENTRYPOINT ["/usr/local/bin/web-server"]
