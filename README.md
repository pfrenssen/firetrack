Firetrack
=========

Web app for budget tracking.


Requirements
------------

* [Rust 2018 edition toolchain](https://www.rust-lang.org/tools/install)
* [PostgreSQL](https://www.postgresql.org/)
* [Diesel CLI](https://github.com/diesel-rs/diesel/tree/master/diesel_cli)
* [Composer](https://getcomposer.org/)
* [Clang](https://clang.llvm.org/)


Installation
------------

### Docker

Copy `docker-compose.override.yml.dist` to `docker-compose.override.yml` and
edit it to match your needs.

```
$ git clone https://github.com/pfrenssen/firetrack.git
$ cd firetrack
$ docker-compose run composer composer install
$ docker-compose build
$ docker-compose run diesel-cli database setup
$ docker-compose up
```

### Native

```
$ git clone https://github.com/pfrenssen/firetrack.git
$ cd firetrack
$ composer install
$ cargo build --release
$ sudo ln -s `pwd`/target/release/cli /usr/local/bin/firetrack
```


Configuration
-------------

The available configuration options are listed in `.env.dist`. You will need to
override certain options like the database credentials.

Configure your local environment by creating a `.env` file and override the
necessary configuration options:

```
# The database connection.
DATABASE_URL=postgres://myuser:mypass@localhost/mydatabasename

# The session key, 32 8-bit integers used as a seed to generate session IDs.
SESSION_KEY=189,154,170,151,17,183,51,68,42,157,147,5,137,119,96,60,118,84,241,175,181,201,132,24,144,134,42,46,63,20,231,255

# The secret key used in password hashing.
SECRET_KEY=mysecret123

# The API key for Mailgun.
MAILGUN_API_KEY=0123456789abcdef0123456789abcdef-01234567-89abcdef

# The Mailgun domain to use for sending notifications.
MAILGUN_USER_DOMAIN=sandbox0123456789abcdef0123456789abcdef.mailgun.org
```


Database setup
--------------

Create a new, empty, PostgreSQL database to host the application data, using the
credentials from the `DATABASE_URL` option above. Then populate the database
tables using the Diesel CLI:

```
# Navigate to the `db` crate.
$ cd db/

# Set up the database tables using the Diesel command line interface.
$ diesel database setup
```


Running tests
-------------

Now Firetrack should be ready to go. In order to see that everything works as
expected, try running the test suites. The main tests can be run from the
project root folder:

```
$ cargo test
```

Before we can run the BDD test suite, we need to set up the test environment:

```
# Start the Mailgun mock server.
$ cargo run -- mailgun-mock-server &> /dev/null &

# Start the Firetrack server, using the mock server for Mailgun. This ensures
# we will not be accessing the real Mailgun API during the test.
$ MAILGUN_API_ENDPOINT=http://localhost:8089 cargo run -- serve &> /dev/null &

# Execute the BDD user scenarios.
$ ./vendor/bin/behat
```


Usage
-----

### Web application

Start the webserver on [http://localhost:8088](http://localhost:8088):

```
$ firetrack serve
```

### Command line interface

```
# Using Docker:
$ ./scripts/firetrack-exec.sh

# Native:
$ firetrack
```
