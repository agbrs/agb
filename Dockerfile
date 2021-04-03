FROM docker.io/devkitpro/devkitarm:20190212

RUN apt-get update && \
    apt-get install -y build-essential && \
    apt-get clean

RUN curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain nightly
RUN . "$HOME/.cargo/env" && \
    rustup component add rust-src

RUN echo 'export PATH=$PATH:$DEVKITARM/bin' >> $HOME/.bashrc

CMD /bin/bash