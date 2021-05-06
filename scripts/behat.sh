#!/bin/bash

# We can use either headless chrome or chromedriver for browser testing.
# Headless chrome is faster and supported by Travis CI, but cannot be inspected
# during the test run. Chromedriver can be launched in a window so the test can
# be followed live. Switch between both using the `javascript_session` option in
# `behat.yml`.

# Start chromedriver.
#chromedriver --port=9222 --url-base=wd/hub &

# Start headless chrome.
chromium --remote-debugging-port=9222 --headless --disable-gpu http://localhost &
CHROMIUM_PID=$!

# Reset database.
cd db/
DATABASE_URL=postgres://pieter@localhost/firetrack-test diesel database reset
cd ..

# Build the project.
cargo build

# Start the Mailgun mock server if it is not already running.
ps aux | grep -v grep | grep mailgun-mock-server &> /dev/null
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

# Kill chromium.
sudo kill -9 $CHROMIUM_PID
