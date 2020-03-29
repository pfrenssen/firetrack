Mailgun mock server
===================

This is a very simple server which mocks the Mailgun API endpoint. It is mainly
intended to be used by the BDD test framework which pretends to be an "external"
user working with the website. It can however also be used during development to
prevent test data from hitting the live Mailgun API.

Every request received by the mock server will return a valid response and will
be logged so that functional tests can check whether the correct Mailgun
notifications are sent out.


Usage
-----

```
# Start the Mailgun mock server.
1 cargo run -- mailgun-mock &> /dev/null &

# Start the Firetrack server, configuring it to use the Mailgun mock server.
$ MAILGUN_API_ENDPOINT=http://localhost:8089 cargo run -- serve &> /dev/null &
```
