# Multi-stage build that includes the Firetrack CLI command inside the Behat
# container so that we can call Firetrack commands inside Behat scenarios.
#FROM rust:1.59-bullseye as firetrack-cli-builder
#RUN apt-get update && apt-get install -y libpq-dev libclang-dev
#WORKDIR /usr/src/firetrack
#COPY . .
#RUN pwd
#RUN ls -al
#RUN cargo install --path cli --locked
#RUN ls /usr/src/firetrack/target/release -al

FROM firetrack-firetrack

# A Composer install on PHP 7.4. Can be replaced with the native docker image
# once our dependencies are updated to PHP 8.1.
FROM php:7.4-cli

WORKDIR /app

RUN apt-get update \
  && apt-get install -y libpq5

# Copy the compiled Firetrack CLI binary over from the main image.
COPY --from=firetrack-firetrack /usr/local/cargo/bin/firetrack-cli .
