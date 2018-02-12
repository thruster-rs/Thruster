# Fanta
## An opinionated framework for web development in rust.

[![Build Status](https://travis-ci.org/trezm/Fanta.svg?branch=master)](https://travis-ci.org/trezm/Fanta)

Fanta is a web framework that aims for developers to be productive and consistent across projects and teams. Its goals are to be:
- Opinionated
- Fast
- Intuitive

Based heavily off of the work here: https://github.com/tokio-rs/tokio-minihttp

## Opinionated

Fanta and Fanta-cli strive to give a good way to do domain driven design. It's also designed to let set you on the right path, but not obfuscate certain hard parts behind libraries.

## Fast

Initial tests of plain-text requests yeilds the following resuts:

```
Framework: Cowboy
Requests/sec:  10982.93
Transfer/sec:      1.36MB
Framework: Node (Koa)
Requests/sec:   9592.82
Transfer/sec:      1.39MB
Framework: Node (Express)
Requests/sec:   5926.87
Transfer/sec:      1.22MB
Framework: Phoenix/Elixir (prod mode)
Requests/sec:    538.02
Transfer/sec:    132.93KB
Framework: Rocket (dev mode)
Requests/sec:   5537.87
Transfer/sec:    789.58KB
Framework: Rocket (prod mode)
Requests/sec:  18555.74
Transfer/sec:      2.58MB
Framework: Fanta (dev mode)
Requests/sec:   7362.82
Transfer/sec:    726.22KB
Framework: Fanta (prod mode)
Requests/sec:  39865.62
Transfer/sec:      3.84MB
```

## Intuitive

Based on frameworks like Koa, and Express, Fanta aims to be a pleasure to develop with.

## Getting Started

### Quick setup with postgres

The easiest way to get started is to install fanta-cli,

```
> cargo install fanta-cli
```

And then to run

```
> fanta-cli init MyAwesomeProject
> fanta-cli component Users
> fanta-cli migrate
```

Which will generate everything you need to get started! Note that this requires a running postgres connection and assumes the following connection string is valid:

```
postgres://postgres@localhost/<Your Project Name>
```

This is all configurable and none of it is exposed to the developer. Check out the docs for [fanta-cli here](https://github.com/trezm/fanta-cli).

### Quick setup without a DB

Head on over to our examples section and copy the `hello_world.rs` file, and the `context.rs` file. These provide a good launching point for a basic app that _only_ provides the router (as well as a few examples of how to do things like serializing JSON.)

You could also run:

```
> mkdir my_awesome_project
> cd my_awesome_project
> cargo init --bin
> wget https://raw.githubusercontent.com/trezm/Fanta/master/examples/hello_world/main.rs
> wget https://raw.githubusercontent.com/trezm/Fanta/master/examples/hello_world/context.rs
```

For basic functionality, you'll want to include: `fanta, tokio_proto, tokio_service, lazy_static`.

For JSON serialization, you'll also want: `serde, serde_json, serde_derive`.

[Simple Plaintext and JSON routes](https://github.com/trezm/Fanta/blob/master/examples/hello_world.rs)
