# syntax=docker/dockerfile:1.6
# Hand-maintained. ferro 0.1.72 `docker:init` output is incomplete for this
# project (no multi-bin support, no copy_dirs, no runtime apt packages).
# See .planning/phases/75-re-scaffold-dockerfile-and-digitalocean-app-spec/75-VERIFICATION.md.
# Do NOT run `ferro docker:init --force` against this file until the upstream
# scaffolder is fixed — it will overwrite with a broken template.

FROM rust:slim-bookworm AS chef
WORKDIR /app
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && cargo install cargo-chef --locked

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS backend-builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*
# ferro:runtime-apt
RUN apt-get update \
    && apt-get install -y --no-install-recommends chromium fonts-liberation ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY --from=backend-builder /app/target/release/gestiscilo /usr/local/bin/gestiscilo
COPY --from=backend-builder /app/target/release/screenshot-worker /usr/local/bin/screenshot-worker
COPY themes themes
EXPOSE 8080
ENTRYPOINT ["/usr/local/bin/gestiscilo"]
CMD ["serve"]
