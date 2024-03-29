FROM rust:1.67 AS builder

RUN rustup toolchain install nightly

COPY Cargo.toml Cargo.lock ./

RUN mkdir ./src \
 && echo "fn main() {}" > ./src/main.rs \
 && cargo +nightly build --release -Z sparse-registry

COPY src/ ./src/
RUN cargo +nightly build --release -Z sparse-registry

FROM debian:bullseye-slim

COPY --from=builder ./target/release/helium-mock-follower /opt/helium_mock_follower/bin/helium-mock-follower

COPY demo_gateways.csv /demo_gateways.csv

EXPOSE 8080

CMD ["/opt/helium_mock_follower/bin/helium-mock-follower", "server"]
