# ==============================================================================
# APPLICATION
# ==============================================================================
# Core application settings

APP_NAME="Ferro Application"
APP_ENV=local          # local, staging, production, testing
APP_DEBUG=true         # Set false in production
APP_URL=http://localhost:8080

# ==============================================================================
# SERVER
# ==============================================================================
# HTTP server binding and limits

SERVER_HOST=127.0.0.1
SERVER_PORT=8080
SERVER_MAX_BODY_SIZE=10485760  # Max request body in bytes (default: 10MB)
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
SESSION_SECURE=true    # Set false for local HTTP development
SESSION_PATH=/
SESSION_SAME_SITE=Lax  # Strict, Lax, or None

# ==============================================================================
# REDIS
# ==============================================================================
# Required for: redis cache driver, redis queue driver, broadcasting
# REDIS_URL takes precedence over individual host/port/password/database vars

REDIS_URL=             # Full URL (e.g., redis://127.0.0.1:6379/0)
REDIS_HOST=127.0.0.1
REDIS_PORT=6379
REDIS_PASSWORD=
REDIS_DATABASE=0

# ==============================================================================
# CACHE
# ==============================================================================
# Drivers: memory (default, no setup), redis
# ferro-cache reads CACHE_DRIVER, CACHE_PREFIX, CACHE_TTL, CACHE_MEMORY_CAPACITY
# framework CacheConfig reads REDIS_URL, REDIS_PREFIX, CACHE_DEFAULT_TTL

CACHE_DRIVER=memory
CACHE_PREFIX=          # Key prefix for ferro-cache entries (default: "")
CACHE_TTL=3600         # Default TTL in seconds for ferro-cache (default: 3600)
CACHE_MEMORY_CAPACITY=10000  # Max entries for memory store (default: 10000)
REDIS_PREFIX=ferro_cache_    # Key prefix for framework CacheConfig (default: "ferro_cache:")
CACHE_DEFAULT_TTL=3600       # Default TTL for framework CacheConfig (default: 3600)

# ==============================================================================
# QUEUE
# ==============================================================================
# Drivers: sync (default, inline processing), redis (background workers)

QUEUE_CONNECTION=sync
QUEUE_DEFAULT=default          # Default queue name
QUEUE_PREFIX=ferro_queue       # Redis key prefix (default: "ferro_queue")
QUEUE_BLOCK_TIMEOUT=5          # Seconds to block waiting for jobs (default: 5)
QUEUE_MAX_CONCURRENT=10        # Max concurrent jobs per worker (default: 10)

# ==============================================================================
# STORAGE
# ==============================================================================
# Disks: local (default), public, s3

FILESYSTEM_DISK=local
FILESYSTEM_LOCAL_ROOT=./storage        # Root path for local disk
FILESYSTEM_LOCAL_URL=                  # Public URL for local files (optional)
FILESYSTEM_PUBLIC_ROOT=./storage/public  # Root path for public disk
FILESYSTEM_PUBLIC_URL=/storage         # Public URL for public files

# ==============================================================================
# AWS S3
# ==============================================================================
# Required when FILESYSTEM_DISK=s3 (needs s3 feature enabled)

AWS_ACCESS_KEY_ID=
AWS_SECRET_ACCESS_KEY=
AWS_DEFAULT_REGION=us-east-1
AWS_BUCKET=
AWS_URL=               # For S3-compatible services (MinIO, DigitalOcean Spaces)

# ==============================================================================
# BROADCASTING
# ==============================================================================
# WebSocket broadcasting configuration

BROADCAST_MAX_SUBSCRIBERS=0    # Per channel, 0 = unlimited (default: 0)
BROADCAST_MAX_CHANNELS=0       # Total channels, 0 = unlimited (default: 0)
BROADCAST_HEARTBEAT_INTERVAL=30  # Seconds (default: 30)
BROADCAST_CLIENT_TIMEOUT=60     # Seconds, disconnect if no activity (default: 60)
BROADCAST_ALLOW_CLIENT_EVENTS=true  # Allow client-to-client whisper (default: true)

# ==============================================================================
# MAIL
# ==============================================================================
# SMTP configuration for notifications

MAIL_DRIVER=smtp       # Used by scaffolded app config
MAIL_HOST=localhost
MAIL_PORT=587
MAIL_USERNAME=
MAIL_PASSWORD=
MAIL_FROM_ADDRESS=hello@example.com
MAIL_FROM_NAME="Ferro App"
MAIL_ENCRYPTION=tls    # "tls" or "none" (default: "tls")

# ==============================================================================
# SLACK
# ==============================================================================
# Slack notification webhook (optional)

SLACK_WEBHOOK_URL=

# ==============================================================================
# DEBUG & METRICS
# ==============================================================================
# Introspection and performance monitoring

FERRO_DEBUG_ENDPOINTS=         # Enable debug routes in production (default: from APP_DEBUG)
FERRO_COLLECT_METRICS=true     # Enable request metrics collection (default: true)

# ==============================================================================
# AI
# ==============================================================================
# For AI-powered CLI features (ferro make:view --ai)

ANTHROPIC_API_KEY=
FERRO_AI_MODEL=claude-sonnet-4-5  # Override AI model (default: "claude-sonnet-4-5")
