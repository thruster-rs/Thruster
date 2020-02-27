# Release notes

Below you'll find a set of notes for migrating between versions. It should be noted that although the intent is to follow semantic versioning, until thruster is released in earnest (1.x), the second digit will be the one that dictates breaking changes.

## 0.9.0

Breaking changes:
- `thruster::thruster_proc::async_middleware -> thruster::async_middleware`
- `thruster::server::Server -> thruster::Server`
- `thruster::thruster_context -> thruster::context`
- You'll have to change middleware that returns a `Context` into middleware that returns a `MiddlewareResult`, so `async fn foo(context: Ctx, next: MiddlewareNext<Ctx>) -> Ctx` becomes ``async fn foo(context: Ctx, next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx>`. In addition, the return value is now a result, so `context` becomes `Ok(context)`.

## 0.8.0

What a time it has been! This release includes a lot of various bug and stability fixes along with a few big things:
- `async_middleware` has been migrated from proc macro to normal macro, which means... **Async/Await thruster is now usable on stable!!!** 🎉🎉🎉
- `thruster_async_await` flag has been removed as now that's the default
- If you're not using async/await, migrate over before making the switch. We will no longer be supporting non-async thruster. Subsequent releases will focus on cleaning up the code a little more.

## 0.7.12

While async_await is still not stable (a month or so to go!) this release disables testing by default since it's not yet supported in an async await format. In order to enable, add the feature `thruster_testing` to your crate configuration.

## 0.7.11

This release adds support for SSL servers via the `thruster::ssl_server::SSLServer` package. The new `SSLServer` works much like the old thruster server, except with the addition of needing to set `cert` and, optionally, `cert_pass` on the server struct.

```rust
  let mut server = SSLServer::new(app);
  server.cert(include_bytes!("identity.p12").to_vec());
  server.cert_pass("asdfasdfasdf");
  server.start("0.0.0.0", 4321);
```

## 0.7.x

This release introduces some great new features, but most importantly it allows thruster to run in async/await mode. Some house cleaning efforts also had to be made in order to enable this, including dividing the repository into work spaces with appropriate feature flags. This means that some large steps have to be made to upgrade from 0.6.x, namely:

```
thruster::builtins::server                ->    thruster::server
thruster::builtins::hyper_server::Server  ->    thruster::server::HyperServer
thruster::builtins::query_params          ->    thruster::thruster_middleware::query_params
thruster::builtins::cookies               ->    thruster::thruster_middleware::cookies
```
