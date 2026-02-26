# ==========================================
# Stage 1: Build Frontend
# ==========================================
FROM node:22-slim AS frontend-builder

WORKDIR /app/frontend

# Install dependencies
COPY frontend/package.json frontend/package-lock.json* ./
RUN npm ci

# Copy frontend source and build
COPY frontend/ ./
RUN npm run build

# ==========================================
# Stage 2: Cargo Chef Base
# ==========================================
FROM rust:1.88-slim-bookworm AS chef

RUN cargo install cargo-chef
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# ==========================================
# Stage 3: Prepare Dependency Recipe
# ==========================================
FROM chef AS planner

COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
RUN cargo chef prepare --recipe-path recipe.json

# ==========================================
# Stage 4: Build Rust Backend
# ==========================================
FROM chef AS backend-builder

COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/
COPY --from=frontend-builder /app/public ./public
RUN cargo build --release

# ==========================================
# Stage 5: Runtime Image
# ==========================================
FROM debian:bookworm-slim AS runtime

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 appuser

# Copy the compiled binary
COPY --from=backend-builder --chown=appuser:appuser /app/target/release/{package_name} /usr/local/bin/app

# Copy public assets and translations
COPY --from=backend-builder --chown=appuser:appuser /app/public ./public
COPY --from=backend-builder --chown=appuser:appuser /app/lang ./lang

# Create storage directory for file uploads
RUN mkdir -p storage/app/public && chown -R appuser:appuser storage

USER appuser

# Environment variables
ENV APP_ENV=production
ENV SERVER_HOST=0.0.0.0
ENV SERVER_PORT=8080

EXPOSE 8080

ENTRYPOINT ["/usr/local/bin/app"]
