# COMP-04 Cold-Cache Benchmark Container
#
# Why debian:bookworm-slim and not rust:slim:
# rust:slim ships a pre-installed Rust toolchain and a pre-warmed Cargo registry
# index. Using it would measure a warm cache, not a cold one. debian:bookworm-slim
# has no toolchain and no Cargo cache — rustup installs from scratch on first build,
# which is the "first-time experience" the benchmark measures.
#
# Note: pinning this image by digest hardens supply-chain (T-211-02). The tag
# debian:bookworm-slim is acceptable for initial runs; replace with a digest-pinned
# reference (FROM debian:bookworm-slim@sha256:<digest>) for reproducible baselines.
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    curl ca-certificates build-essential git \
    && rm -rf /var/lib/apt/lists/*

# Install Rust toolchain via rustup.
# --proto '=https' --tlsv1.2 pins transport and TLS version (T-211-01 mitigation).
# --profile minimal keeps the install lean (no rustfmt/clippy in the timed container).
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --default-toolchain stable --profile minimal

ENV PATH="/root/.cargo/bin:${PATH}"

# Install the ferro CLI from crates.io in a Docker layer (cached after first build).
# This step is NOT part of the timed benchmark sequence — the benchmark measures
# project scaffolding time, not CLI installation time. The ferro-cli version
# installed here should match the ferro-rs version the scaffolded project will
# resolve from crates.io during cargo build.
RUN cargo install ferro-cli

WORKDIR /bench

# cold cache != network isolation: cargo build in the timed sequence resolves
# ferro-rs and its transitive dependencies from crates.io, which requires network.
# "Cold" means no Cargo registry cache pre-warmed and no toolchain pre-installed —
# not network isolation. Do NOT run with --network none.
CMD ["bash", "-c", "\
  echo '=== COMP-04 Cold-Cache Benchmark ===' && \
  SECONDS=0 && T0=$SECONDS && \
  ferro new bench-app --no-interaction --no-git && \
  echo \"Step 1 ferro new: $((SECONDS - T0))s\" && T0=$SECONDS && \
  cd bench-app && \
  ferro make:auth && \
  echo \"Step 2 make:auth: $((SECONDS - T0))s\" && T0=$SECONDS && \
  ferro make:scaffold Article title:string body:text --no-smart-defaults -q -y --api && \
  echo \"Step 3a make:scaffold Article: $((SECONDS - T0))s\" && T0=$SECONDS && \
  ferro make:scaffold Product name:string price:float --no-smart-defaults -q -y --api && \
  echo \"Step 3b make:scaffold Product: $((SECONDS - T0))s\" && T0=$SECONDS && \
  ferro make:scaffold Order status:string total:float --no-smart-defaults -q -y --api && \
  echo \"Step 3c make:scaffold Order: $((SECONDS - T0))s\" && T0=$SECONDS && \
  ferro make:job EmailNotification && \
  echo \"Step 4 make:job: $((SECONDS - T0))s\" && T0=$SECONDS && \
  cargo build && \
  echo \"Step 5 cargo build: $((SECONDS - T0))s\" \
"]
