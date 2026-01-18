APP_NAME="{project_name}"
APP_ENV=local
APP_DEBUG=true
APP_URL=http://localhost:8080

SERVER_HOST=127.0.0.1
SERVER_PORT=8080

VITE_PORT=5173

# Build cleanup: auto-remove artifacts older than N days on `ferro serve`
# Set to 0 to disable automatic cleanup (requires cargo-sweep)
CARGO_SWEEP_DAYS=7

# Database (SQLite by default, change to postgres://user:pass@localhost:5432/dbname for PostgreSQL)
DATABASE_URL=sqlite://./database.db
DB_MAX_CONNECTIONS=10
DB_MIN_CONNECTIONS=1
DB_CONNECT_TIMEOUT=30
DB_LOGGING=false

# Session
SESSION_LIFETIME=120
SESSION_COOKIE=ferro_session
SESSION_SECURE=false
SESSION_PATH=/
SESSION_SAME_SITE=Lax

# Redis (for cache, queue, and broadcasting)
REDIS_HOST=127.0.0.1
REDIS_PORT=6379
REDIS_PASSWORD=
REDIS_DATABASE=0

# Cache
CACHE_DRIVER=memory
CACHE_PREFIX=ferro_cache_

# Queue
QUEUE_CONNECTION=sync
QUEUE_DEFAULT=default
QUEUE_RETRY_AFTER=90

# Storage
FILESYSTEM_DISK=local
STORAGE_PATH=storage/app

# AWS S3 (optional, for s3 disk driver)
AWS_ACCESS_KEY_ID=
AWS_SECRET_ACCESS_KEY=
AWS_DEFAULT_REGION=us-east-1
AWS_BUCKET=
AWS_ENDPOINT=

# Broadcasting
BROADCAST_DRIVER=log
PUSHER_APP_ID=
PUSHER_APP_KEY=
PUSHER_APP_SECRET=
PUSHER_APP_CLUSTER=mt1

# Mail
MAIL_DRIVER=smtp
MAIL_HOST=localhost
MAIL_PORT=587
MAIL_USERNAME=
MAIL_PASSWORD=
MAIL_FROM_ADDRESS=hello@example.com
MAIL_FROM_NAME="Ferro App"
