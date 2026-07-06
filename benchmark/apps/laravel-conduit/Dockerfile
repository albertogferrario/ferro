# benchmark/apps/laravel-conduit/Dockerfile
# Vendored f1amy/laravel-realworld-example-app (Laravel 9, PHP 8.x) served as the
# Conduit php-fpm baseline. Serving stack and pool config are identical to
# benchmark/apps/laravel-micro so the micro and Conduit numbers are comparable.
# Pool: pm=static, max_children=20, opcache enabled (see docker/php-fpm.conf).
# PHP 8.2 (not 8.3): the vendored app pins an older nesbot/carbon that is
# incompatible with PHP 8.3's DateTime::getLastErrors() returning false
# (Carbon::setLastErrors TypeError). 8.2 is within the app's ^8.0.2 constraint.
FROM php:8.2-fpm-bookworm
RUN apt-get update && apt-get install -y --no-install-recommends \
        libpq-dev unzip nginx supervisor \
    && docker-php-ext-install pdo_pgsql opcache \
    && rm -rf /var/lib/apt/lists/*

# opcache tuning — standard production settings (identical to laravel-micro)
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
# Vendored community app: install its locked deps (no app-logic changes).
RUN composer install --no-dev --optimize-autoloader --no-interaction
# Config-only dependency bump: the locked nesbot/carbon 2.58.0 is incompatible with
# modern PHP's DateTime::getLastErrors() returning false (Carbon::setLastErrors
# TypeError). Bump within the ^2 line to a patched release. App logic unchanged.
RUN composer update nesbot/carbon --with-all-dependencies --no-dev \
        --optimize-autoloader --no-interaction

# nginx: remove default site, install ours
RUN rm -f /etc/nginx/sites-enabled/default
COPY docker/nginx.conf /etc/nginx/sites-enabled/conduit.conf

# php-fpm: install pool config (replaces the default www.conf)
RUN rm -f /usr/local/etc/php-fpm.d/www.conf
COPY docker/php-fpm.conf /usr/local/etc/php-fpm.d/www.conf

# supervisord + migrate-on-boot entrypoint
COPY docker/supervisord.conf /etc/supervisor/conf.d/supervisord.conf
COPY docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

# storage must be writable by www-data (fpm runs as www-data)
RUN mkdir -p /var/run/php storage/logs storage/framework/cache \
             storage/framework/sessions storage/framework/views bootstrap/cache \
    && chown -R www-data:www-data storage bootstrap/cache

ENV APP_ENV=production
EXPOSE 8000

CMD ["/usr/local/bin/entrypoint.sh"]
