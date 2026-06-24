FROM rust:1.96.0-bookworm AS builder

RUN apt-get update && apt-get install docker.io -y --no-install-recommends

RUN mkdir /build
WORKDIR /build
ADD Cargo.toml /build
ADD src /build/src

RUN cargo build --release

FROM debian:trixie-20260623

RUN set -eux; \
    apt-get update; \
    DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends ca-certificates; \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/bin/docker /usr/bin/docker
COPY --from=builder /build/target/release/play /bin/play

EXPOSE 5409

CMD ["/bin/play"]
