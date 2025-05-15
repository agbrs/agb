FROM rust:1.87.0-bookworm

RUN cargo install agb-gbafix

RUN groupadd -r appgroup && useradd -r -g appgroup appuser && mkdir /agb && chown appuser:appgroup /agb
WORKDIR /agb

COPY --chown=appuser:appgroup ./template /agb

RUN mkdir -p /home/appuser/cargo && chmod -R 755 /home/appuser && chown -R appuser:appgroup /home/appuser

ENV \
    CARGO_HOME=/home/appuser/cargo \
    PATH=$PATH:/home/appuser/cargo/bin

RUN su appuser -c "cargo add agb_tracker && cargo build"

COPY --chown=appuser:appgroup ./execute.sh /agb/execute.sh

RUN mkdir /out && chown appuser:appgroup /out && chown -R appuser:appgroup /agb

USER appuser

ENTRYPOINT ["./execute.sh"]