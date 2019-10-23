Firetrack
=========

Web app for budget tracking.


Requirements
------------

* [Rust 2018 edition toolchain](https://www.rust-lang.org/tools/install)


Installation
------------

```
$ git clone https://github.com/pfrenssen/firetrack.git
$ cd firetrack
$ cargo build --release
$ sudo ln -s ./target/release/firetrack /usr/local/bin/
```


Usage
-----

Start the webserver on [http://localhost:8088](http://localhost:8088):

```
$ firetrack serve
```
