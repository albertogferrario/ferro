# ==============================================================================
# APPLICATION
# ==============================================================================
# Core application settings

APP_NAME="Ferro Application"
APP_ENV=local          # local, staging, production
APP_DEBUG=true         # Set false in production
APP_URL=http://localhost:8080

# ==============================================================================
# SERVER
# ==============================================================================
# Development server binding

SERVER_HOST=127.0.0.1
SERVER_PORT=8080
VITE_PORT=5173

# ==============================================================================
# BUILD CLEANUP
# ==============================================================================
# Auto-remove build artifacts on `ferro serve` (requires cargo-sweep)

CARGO_SWEEP_DAYS=7     # Set 0 to disable

# ==============================================================================
# DATABASE
# ==============================================================================
# SQLite works out of the box. For PostgreSQL:
# DATABASE_URL=postgres://user:pass@localhost:5432/dbname

DATABASE_URL=sqlite://./database.db
DB_MAX_CONNECTIONS=10
DB_MIN_CONNECTIONS=1
DB_CONNECT_TIMEOUT=30
DB_LOGGING=false       # Set true to see SQL queries

# ==============================================================================
# SESSION
# ==============================================================================
# Session configuration for authentication

SESSION_LIFETIME=120   # Minutes
SESSION_COOKIE=ferro_session
SESSION_SECURE=false   # Set true when using HTTPS
SESSION_PATH=/
SESSION_SAME_SITE=Lax  # Strict, Lax, or None

# ==============================================================================
# REDIS
# ==============================================================================
# Required for: redis cache driver, redis queue driver, redis broadcasting

REDIS_HOST=127.0.0.1
REDIS_PORT=6379
REDIS_PASSWORD=
REDIS_DATABASE=0

# ==============================================================================
# CACHE
# ==============================================================================
# Drivers: memory (default, no setup), file, redis

CACHE_DRIVER=memory
CACHE_PREFIX=ferro_cache_

# ==============================================================================
# QUEUE
# ==============================================================================
# Drivers: sync (default, inline), redis (background workers)

QUEUE_CONNECTION=sync
QUEUE_DEFAULT=default
QUEUE_RETRY_AFTER=90

# ==============================================================================
# STORAGE
# ==============================================================================
# Disks: local (default), s3

FILESYSTEM_DISK=local
STORAGE_PATH=storage/app

# ==============================================================================
# AWS S3
# ==============================================================================
# Required when FILESYSTEM_DISK=s3

AWS_ACCESS_KEY_ID=
AWS_SECRET_ACCESS_KEY=
AWS_DEFAULT_REGION=us-east-1
AWS_BUCKET=
AWS_ENDPOINT=          # For S3-compatible services (MinIO, DigitalOcean Spaces)

# ==============================================================================
# BROADCASTING
# ==============================================================================
# Drivers: log (default, for debugging), redis, pusher

BROADCAST_DRIVER=log
PUSHER_APP_ID=
PUSHER_APP_KEY=
PUSHER_APP_SECRET=
PUSHER_APP_CLUSTER=mt1

# ==============================================================================
# MAIL
# ==============================================================================
# Drivers: smtp, log (for development)

MAIL_DRIVER=smtp
MAIL_HOST=localhost
MAIL_PORT=587
MAIL_USERNAME=
MAIL_PASSWORD=
MAIL_FROM_ADDRESS=hello@example.com
MAIL_FROM_NAME="Ferro App"

# ==============================================================================
# AI
# ==============================================================================
# For AI-powered features

ANTHROPIC_API_KEY=
