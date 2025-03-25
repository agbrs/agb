FROM rust:1.85.1-bullseye AS builder

RUN apt-get update && apt-get install docker.io -y --no-install-recommends

RUN mkdir /build
WORKDIR /build
ADD Cargo.toml /build
ADD src /build/src

RUN cargo build --release

FROM debian:bullseye

COPY --from=builder /usr/bin/docker /usr/bin/docker
COPY --from=builder /build/target/release/play /bin/play

EXPOSE 5409

CMD ["/bin/play"]

#docker run cloudflare/cloudflared:latest tunnel --no-autoupdate run --token eyJhIjoiMzRmNTUzN2IwMTMzNmNlZjY2ZjllNmFiYTczOTAwM2MiLCJ0IjoiYmNmODQ5NjItMGQ4Yy00NjZlLTg4MWMtZjQ5N2JhMmYxZDE3IiwicyI6Ik9EazRPVFJrTlRjdE9EaGpaUzAwTnprNExXSTJNVFV0WXpabU9HTTROall4TjJKayJ9
# laughing_ganguly