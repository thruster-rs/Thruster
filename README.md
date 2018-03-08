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

Using the following command, we get roughly 96% of the speed of pure `tokio-minihttp` running in release mode.

```
wrk -t12 -c400 -d30s http://127.0.0.1:4321/plaintext
```

Fanta results:
```
Running 30s test @ http://127.0.0.1:4321/plaintext
  12 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     6.45ms    1.09ms  18.11ms   75.96%
    Req/Sec     5.03k   502.49     7.75k    82.97%
  1802773 requests in 30.05s, 244.13MB read
  Socket errors: connect 0, read 238, write 0, timeout 0
Requests/sec:  60000.61
Transfer/sec:      8.13MB
```

tokio-minihttp
```
Running 30s test @ http://127.0.0.1:4321/plaintext
  12 threads and 400 connections
  Thread Stats   Avg      Stdev     Max   +/- Stdev
    Latency     6.30ms    2.14ms  52.22ms   88.68%
    Req/Sec     5.19k     1.03k   10.91k    77.92%
  1861702 requests in 30.05s, 229.03MB read
  Socket errors: connect 0, read 243, write 0, timeout 0
Requests/sec:  61949.84
Transfer/sec:      7.62MB
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

### Changelog

#### 0.2.0
* Breaking Changes
  * Migrated to use Futures for all middleware callbacks
* Highlights
  * Dropped support for tree-based lookup of routes to remove potential branching
  * Removed many regexes involved in lookup
