#!/bin/bash
set -ex

# Create a .env file with test values.
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
if [ ! -f "${SCRIPT_DIR}/../.env" ]; then
  # Generate comma-separated list of 32 integers in the range of 0-255.
  SESSION_KEY=$(shuf -i 0-255 -n 32 | tr '\n' ',' | sed 's/,$//')

  # Generate random string of 32 characters for secret key.
  SECRET_KEY=$(head /dev/urandom | tr -dc A-Za-z0-9 | head -c 32)

  # Write content to file.
  cat << EOF > .env
# Some random keys to use in testing.
SESSION_KEY=$SESSION_KEY
SECRET_KEY=$SECRET_KEY

RUST_BACKTRACE=1

# Fast, insecure hashing, only good for testing.
HASHER_MEMORY_SIZE=512
HASHER_ITERATIONS=2
EOF
fi

# Kill any previously running instances.
docker-compose down

# Prepare containers.
#docker-compose build firetrack --progress=plain
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
