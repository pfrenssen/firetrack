# A Composer install on PHP 7.4. Can be replaced with the native docker image
# once our dependencies are updated to PHP 8.1.
FROM php:7.4-cli
COPY --from=composer:2 /usr/bin/composer /usr/bin/composer

RUN apt-get update\
      && apt-get install -y libzip-dev zip unzip git\
      && docker-php-ext-configure zip \
      && docker-php-ext-install zip

WORKDIR /app

RUN composer --version && php -v
