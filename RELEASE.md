# Release notes

Below you'll find a set of notes for migrating between versions. It should be noted that although the intent is to follow semantic versioning, until thruster is released in earnest (1.x), the second digit will be the one that dictates breaking changes.

## 1.1.4

Quick release, realized that multiple pieces of middleware weren't working, e.g. `[profile, plaintext]`. Now we have the fix in!

## 1.1.3

Minor release! Add a few convenience message like `json` on `BasicContext` and fix a bug in the redirect code for `BasicContext` as well.

## 1.1.2

The _entire parser was rebuilt from scratch_. That might not really mean much, but it amounted to a roughly 8% increase in speed across the board! It also means the code backing it is much less of a rat's nest and much more maintainable -- so really a win-win all around.

Since the parser got rewritten, we were able to make the `async_middleware` macros more concise too! In this version, the old `async_middleware` will still work the same, but actually it's just a wrapper around the more simple `m![]` middleware now. The usage is much more straightfoward, as you don't have to provide the context type.

Before:
```rs
app.get("/plaintext", async_middleware!(Ctx, [plaintext]));
```

After:
```rs
app.get("/plaintext", m![plaintext]);
```

All of the dependencies were also upgraded. This, unfortunately, means that when attempting to update the TechEmpower benchmarks, everything broke. If anyone wants to lend a helping hand with getting them back up and tuned correctly, please send a message on Discord!

## 1.0.0

My, my, how time flies! I'd like to introduce everyone to thruster 1.0. I have been using thruster in my projects exclusively for the past year, and finally believe that it is stable enough for a 1.0 release. Contained within the official 1.0 release (since the last release,) you'll find the changes listed below. Some of my favorite highlights:

### Adding gRPC support via tonic

That's right! Thanks to [tonic](https://github.com/hyperium/tonic), we now have support for gRPC. In addition, there's a [crate-in-the-making](https://github.com/thruster-rs/thruster-grpc) to get lower access into the gRPC stack via [PROST](https://github.com/danburkert/prost)! gRPC is great for interacting between microservices, or even efficiently communicating with mobile apps.

### Addings Socket.io support

[Socket.io](https://github.com/thruster-rs/thruster-socketio) is here for all of your real-time websocket communication needs. Although still in its infancy, the current crate has enough capabilities to work via websockets, and even across multiple servers via redis pub-sub.

### Add the concept of a global shared config state

An often asked for feature was the addition of a shared config state throughout a thruster app. I'm happy to announce that this is now available, and even has a provided context for hyper to make your life a little easier. This is great for passing database client pools through the middleware and reducing on the cognitive load of lazy statics. Check out the [using_state](https://github.com/thruster-rs/Thruster/blob/master/thruster/examples/using_state.rs) example for more info.

### Still on the horizon

- I'd like to get back to performance, as we've fallen a bit in the benchmarks. Since we've made a shift to focus on hyper integration, making sure the hyper server is running as efficiently as possible will be a huge part of this.
- Expanding the amount of supplied middleware will be important for consistency for developers choosing thruster. Having a batteries included solution is a great way to get up and running quickly.
- It would be great to have more articles and blogs about using thruster and some common use cases.

*Bug Fixes*
bug: Fix ssl hyper server to work with http2 (#163)
bug: Method agnostic middleware fails in non root cases (#154)

*Features*
feat: Upgrade hyper_ssl_server to use rustls
feat: Make return type of hyper testing generic
feat: Add testing harness for hyper (#161)
feat: Allow typed contexts to be created without request (#160)
feat: Add gRPC example (#159)
feat: Add state and matched route (#152)
feat: Added basic JWT context with example. (#149)
feat: Add unix hyper server (#150)

*Chores*
chore: Bump version
chore: Update thruster version (#155)
chore: Change UPDATE to PATCH
chore: Add documentation for the Context trait (#148)
chore: Update hyper context to avoid using parts (#146)
chore: Bump versions to 0.9.0-alpha.4
chore: Add more helpful EOC message (#144)

## 0.9.0

Breaking changes:
- `thruster::thruster_proc::async_middleware -> thruster::async_middleware`
- `thruster::server::Server -> thruster::Server`
- `thruster::thruster_context -> thruster::context`
- You'll have to change middleware that returns a `Context` into middleware that returns a `MiddlewareResult`, so `async fn foo(context: Ctx, next: MiddlewareNext<Ctx>) -> Ctx` becomes ``async fn foo(context: Ctx, next: MiddlewareNext<Ctx>) -> MiddlewareResult<Ctx>`. In addition, the return value is now a result, so `context` becomes `Ok(context)`.
- No more need for `thruster::MiddlewareReturnValue` in imports!

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
