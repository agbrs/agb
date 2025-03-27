FROM rust:1.85.1-bookworm AS builder

RUN apt-get update && apt-get install docker.io -y --no-install-recommends

RUN mkdir /build
WORKDIR /build
ADD Cargo.toml /build
ADD src /build/src

RUN cargo build --release

FROM debian:bookworm

COPY --from=builder /usr/bin/docker /usr/bin/docker
COPY --from=builder /build/target/release/play /bin/play

EXPOSE 5409

CMD ["/bin/play"]
