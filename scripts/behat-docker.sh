#!/bin/bash

set -ex

# Kill any previously running instances.
docker-compose down

# Prepare containers.
docker-compose run composer composer install
#docker-compose build --progress=plain
docker-compose run diesel-cli database setup

# Reset database.
docker-compose run diesel-cli database reset

# Start containers.
docker-compose up -d

# Wait until the server is up.
until curl -s localhost:8088; do sleep 1; done > /dev/null

# Run Behat tests.
docker-compose run behat ./vendor/bin/behat -vvv --stop-on-failure

# Spin down containers.
docker-compose down
