#!/bin/sh
# Conduit Laravel baseline entrypoint (Octane + RoadRunner variant).
# Config-only: waits for Postgres, caches config, runs migrations, then serves.
set -e

echo "waiting for db ${DB_HOST}:${DB_PORT} ..."
until php -r "exit(@fsockopen(getenv('DB_HOST'), (int)getenv('DB_PORT')) ? 0 : 1);"; do
  sleep 1
done

# Ensure the octane config is published (octane:install during build can be a no-op
# if the package class check ran before the rr binary was present). Idempotent.
php artisan vendor:publish --provider="Laravel\\Octane\\OctaneServiceProvider" --tag=config --force 2>/dev/null || true

# Make the rr binary discoverable on PATH for octane's roadrunner detection.
[ -f /app/rr ] && ln -sf /app/rr /usr/local/bin/rr

php artisan config:cache
php artisan migrate --force

exec php artisan octane:start --server=roadrunner \
    --host=0.0.0.0 --port=8080 --workers=16 --rpc-port=6001
