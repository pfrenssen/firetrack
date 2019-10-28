Firetrack
=========

Web app for budget tracking.


Requirements
------------

* [Rust 2018 edition toolchain](https://www.rust-lang.org/tools/install)
* [PostgreSQL](https://www.postgresql.org/)


Installation
------------

```
$ git clone https://github.com/pfrenssen/firetrack.git
$ cd firetrack
$ cargo build --release
$ sudo ln -s ./target/release/firetrack /usr/local/bin/
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

# The secret key used in password hashing.
SECRET_KEY=mysecret123
```


Usage
-----

Start the webserver on [http://localhost:8088](http://localhost:8088):

```
$ firetrack serve
```
