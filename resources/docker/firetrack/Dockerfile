# Todo:
# - Move the 'serve' CMD argument to docker-compose.yml. We should be able to
#   use this container as an executable.
# - The vendor folder is copied into the container, which is slow. We could
#   ignore this folder and make the assets available through a volume, but does
#   this make it slower to access the assets?
# - At the moment when building the container all cargo dependencies need to be
#   compiled from scratch. Allow to do incremental compilation.

FROM rust:1.59-bullseye

RUN apt-get update && apt-get install -y libpq-dev libclang-dev

WORKDIR /usr/src/firetrack

COPY . .

RUN cargo fetch
# Compile Firetrack and install the CLI tool at
# /usr/local/cargo/bin/firetrack-cli
RUN cargo install --path cli --locked

ENTRYPOINT ["firetrack-cli"]
CMD ["serve"]
