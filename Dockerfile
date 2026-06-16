FROM rust:1.96-slim AS builder

WORKDIR /build

COPY . .

RUN cargo build --release --locked -p server

FROM debian:trixie-slim AS runtime

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --system --create-home --uid 10001 quic
USER quic

WORKDIR /app
COPY --from=builder /build/target/release/server /app/server

EXPOSE 5000/udp

ENTRYPOINT ["/app/server"]
