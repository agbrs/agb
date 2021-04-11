FROM docker.io/ubuntu:latest

RUN apt-get update && \
    apt-get install -y build-essential binutils-arm-none-eabi curl && \
    apt-get clean

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly
RUN . "$HOME/.cargo/env" && \
    rustup component add rust-src

CMD /bin/bash