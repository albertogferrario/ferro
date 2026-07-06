# benchmark/apps/laravel-micro/Dockerfile
# Serving: php-fpm 8.3 + nginx (via supervisord) — production-representative stack.
# Pool: pm=static, max_children=20, opcache enabled (see docker/php-fpm.conf).
FROM php:8.3-fpm-bookworm
RUN apt-get update && apt-get install -y --no-install-recommends \
        libpq-dev unzip nginx supervisor \
    && docker-php-ext-install pdo_pgsql opcache \
    && rm -rf /var/lib/apt/lists/*

# opcache tuning — standard production settings
RUN { \
    echo "opcache.enable=1"; \
    echo "opcache.memory_consumption=128"; \
    echo "opcache.interned_strings_buffer=8"; \
    echo "opcache.max_accelerated_files=10000"; \
    echo "opcache.revalidate_freq=0"; \
    echo "opcache.validate_timestamps=0"; \
    echo "opcache.fast_shutdown=1"; \
} > /usr/local/etc/php/conf.d/opcache.ini

COPY --from=composer:2.7 /usr/bin/composer /usr/bin/composer
WORKDIR /app
COPY . .
RUN composer install --no-dev --optimize-autoloader

# nginx: remove default site, install ours
RUN rm -f /etc/nginx/sites-enabled/default
COPY docker/nginx.conf /etc/nginx/sites-enabled/benchmark.conf

# php-fpm: install pool config (replaces the default www.conf)
RUN rm -f /usr/local/etc/php-fpm.d/www.conf
COPY docker/php-fpm.conf /usr/local/etc/php-fpm.d/www.conf

# supervisord
COPY docker/supervisord.conf /etc/supervisor/conf.d/supervisord.conf

# storage must be writable by www-data (fpm runs as www-data)
RUN mkdir -p /var/run/php storage/logs storage/framework/cache \
             storage/framework/sessions storage/framework/views bootstrap/cache \
    && chown -R www-data:www-data storage bootstrap/cache

ENV APP_ENV=production
EXPOSE 8000

# Cache config at container start so DB_HOST from compose env is available
CMD ["sh", "-c", \
    "php artisan config:cache && \
     supervisord -c /etc/supervisor/conf.d/supervisord.conf"]
