#!/bin/bash

# Start Selenium.
docker run -d -p 4444:4444 --name=firetrack_selenium --rm --network=host selenium/standalone-chrome:3.141.59-20210929

# Reset database.
cd db/ || exit 1
DATABASE_URL=postgres://pieter@localhost/firetrack-test diesel database reset
cd ..

# Build the project.
cargo build

# Start the Mailgun mock server if it is not already running.
pgrep -f mailgun-mock-server &> /dev/null
if [ $? -eq 1 ]; then
  echo "Mailgun mock server not running. Starting."
  cargo run -- mailgun-mock-server > /dev/null 2>&1 &
fi

# Start the Firetrack server.
DATABASE_URL=postgres://pieter@localhost/firetrack-test MAILGUN_API_ENDPOINT=http://localhost:8089 PORT=8090 cargo run -- serve > /dev/null 2>&1  &
FIRETRACK_SERVER_PID=$!

# Wait until the server is up.
until curl -s localhost:8090; do sleep 1; done > /dev/null

# Run Behat tests.
DATABASE_URL=postgres://pieter@localhost/firetrack-test ./vendor/bin/behat -vvv

# Kill running servers after finishing tests:
sudo kill -9 $FIRETRACK_SERVER_PID

# Kill Selenium.
docker stop firetrack_selenium
