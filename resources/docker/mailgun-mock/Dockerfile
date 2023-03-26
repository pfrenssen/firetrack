FROM firetrack-firetrack

FROM debian:bullseye-slim

WORKDIR /usr/src/firetrack/

RUN apt-get update \
  && apt-get install libpq5 -y

# Copy the compiled Firetrack CLI binary over from the main image.
COPY --from=firetrack-firetrack /usr/local/cargo/bin/firetrack-cli .

ENTRYPOINT ["./firetrack-cli"]
CMD ["mailgun-mock-server"]
