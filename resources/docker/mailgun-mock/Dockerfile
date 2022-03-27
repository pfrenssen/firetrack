# Todo:
# - We are rebuilding the entire Firetrack binary, reuse the standard one.

FROM rust:1.59-bullseye

RUN apt-get update && apt-get install -y libpq-dev libclang-dev

WORKDIR /usr/src/firetrack

COPY . .

RUN cargo install --path cli --locked

ENTRYPOINT ["firetrack-cli"]
CMD ["mailgun-mock-server"]
