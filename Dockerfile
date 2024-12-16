FROM rust:1.83.0-bookworm AS build

WORKDIR /infomaniak-dyndns-wildcard

COPY ./Cargo.lock ./Cargo.lock
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src

RUN cargo build --release

FROM debian:12.8-slim

RUN apt-get update && \
    apt-get install -y ca-certificates openssl && \
    rm -rf /var/lib/apt/lists/*

COPY --from=build /infomaniak-dyndns-wildcard/target/release/infomaniak-dyndns-wildcard /usr/local/bin

CMD ["/usr/local/bin/infomaniak-dyndns-wildcard"]
