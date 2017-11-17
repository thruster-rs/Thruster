# Fanta
## An opinionated framework for web development in rust.

[![Build Status](https://travis-ci.org/trezm/Fanta.svg?branch=master)](https://travis-ci.org/trezm/Fanta)

Fanta is a web framework that aims for developers to be productive and consistent across projects and teams. Its goals are to be:
- Opinionated
- Fast
- Intuitive

Based heavily off of the work here: https://github.com/tokio-rs/tokio-minihttp

## WARNING -- FANTA IS NOT READY, AND PROBABLY SHOULDN'T EVEN BE ON GITHUB YET

## Opinionated

This is a work in progress

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

Left to the developer to decide at this point!
