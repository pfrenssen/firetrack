FROM rust:1.59-bullseye

RUN apt-get update\
      && apt-get install -y libpq-dev libclang-dev

RUN cargo install diesel_cli --no-default-features --features postgres

WORKDIR /usr/src/firetrack/db

COPY db .

ENTRYPOINT ["diesel"]
