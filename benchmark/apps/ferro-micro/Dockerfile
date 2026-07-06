# benchmark/apps/ferro-micro/Dockerfile
FROM rust:1.88.0-slim-bookworm AS build
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev \
 && rm -rf /var/lib/apt/lists/*
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libpq5 \
 && rm -rf /var/lib/apt/lists/*
COPY --from=build /app/target/release/ferro-micro /usr/local/bin/app
# Server host/port are env vars (SERVER_HOST, SERVER_PORT), not CLI flags on the binary
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=3000
EXPOSE 3000
CMD ["/usr/local/bin/app", "serve"]
