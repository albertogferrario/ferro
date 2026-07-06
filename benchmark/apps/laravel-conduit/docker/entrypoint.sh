#!/bin/sh
# Conduit Laravel baseline entrypoint (php-fpm + nginx variant).
# Config-only: waits for Postgres, caches config, runs migrations, then serves.
set -e

# Wait for the shared Postgres to accept connections (compose depends_on covers
# container health, but the DB may still be initializing the `conduit` database).
echo "waiting for db ${DB_HOST}:${DB_PORT} ..."
until php -r "exit(@fsockopen(getenv('DB_HOST'), (int)getenv('DB_PORT')) ? 0 : 1);"; do
  sleep 1
done

php artisan config:cache
php artisan migrate --force

exec supervisord -c /etc/supervisor/conf.d/supervisord.conf
