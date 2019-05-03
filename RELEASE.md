# Release notes

Below you'll find a set of notes for migrating between versions. It should be noted that although the intent is to follow semantic versioning, until thruster is released in earnest (1.x), the second digit will be the one that dictates breaking changes.

## 0.7.x

This release introduces some great new features, but most importantly it allows thruster to run in async/await mode. Some house cleaning efforts also had to be made in order to enable this, including dividing the repository into work spaces with appropriate feature flags. This means that some large steps have to be made to upgrade from 0.6.x, namely:

```
thruster::builtins::server                ->    thruster::server
thruster::builtins::hyper_server::Server  ->    thruster::server::HyperServer
thruster::builtins::query_params          ->    thruster::thruster_middleware::query_params
thruster::builtins::cookies               ->    thruster::thruster_middleware::cookies
```
