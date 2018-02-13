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

### Quick setup without a DB

The easiest way to get started is to just clone the [starter kit](https://github.com/trezm/fanta-starter-kit)

```
> git clone git@github.com:trezm/fanta-starter-kit.git
> cd fanta-starter-kit
> cargo run
```

The example provides a simple route plaintext route, a route with JSON serialization, and the preferred way to organize sub routes using sub apps.

### Quick setup with postgres

The easiest way to get started with postgres is to install fanta-cli,

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
