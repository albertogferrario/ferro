# syntax=docker/dockerfile:1.6
#
# Multi-stage image for the Nearly reference app.
#
# Build from the REPO ROOT (nearly depends on ~20 workspace crates by path):
#   docker build -f nearly/Dockerfile -t nearly .
# Run:
#   docker run --rm -p 8080:8080 nearly
#
# SERVER_HOST=0.0.0.0 makes the server reachable from outside the container.

# 1) Frontend — Vite build. `outDir: '../public'` writes to /app/public.
FROM node:20-slim AS frontend
WORKDIR /app/frontend
COPY nearly/frontend/package.json nearly/frontend/package-lock.json ./
RUN npm ci
COPY nearly/frontend/ ./
RUN npm run build

# 2) Backend — build the release binary from the workspace.
FROM rust:1.88-slim AS backend
WORKDIR /src
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*
COPY . .
RUN cargo build --release -p nearly

# 3) Runtime — slim image with the binary + built frontend assets.
FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && mkdir -p /app/data
COPY --from=backend /src/target/release/nearly /usr/local/bin/nearly
# The server reads public/.vite/manifest.json + serves /assets/* relative to CWD.
COPY --from=frontend /app/public ./public
ENV APP_ENV=production \
    APP_NAME=Nearly \
    APP_URL=http://localhost:8080 \
    SERVER_HOST=0.0.0.0 \
    SERVER_PORT=8080 \
    DATABASE_URL=sqlite:///app/data/nearly.db
EXPOSE 8080
CMD ["nearly"]
