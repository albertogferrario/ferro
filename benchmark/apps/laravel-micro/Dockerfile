# benchmark/apps/laravel-micro/Dockerfile
FROM php:8.3-cli-bookworm
RUN apt-get update && apt-get install -y --no-install-recommends libpq-dev unzip \
 && docker-php-ext-install pdo_pgsql opcache && rm -rf /var/lib/apt/lists/*
COPY --from=composer:2.7 /usr/bin/composer /usr/bin/composer
WORKDIR /app
COPY . .
RUN composer install --no-dev --optimize-autoloader
ENV APP_ENV=production
EXPOSE 8000
# Cache config at startup so DB_HOST from compose env is baked before artisan serve forks workers.
# Note: stock artisan serve; Octane variant is a documented future addition (see benchmark/README.md)
CMD ["sh", "-c", "php artisan config:cache && php artisan serve --host=0.0.0.0 --port=8000"]
