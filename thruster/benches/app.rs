extern crate bytes;
extern crate futures;

#[macro_use]
extern crate criterion;
#[macro_use]
extern crate thruster;

use bytes::{BufMut, BytesMut};
use futures::{future, Future};
use std::boxed::Box;
use std::collections::HashMap;

use criterion::Criterion;
use thruster::thruster_middleware::query_params::query_params;
use thruster::{decode, App, BasicContext, MiddlewareChain, MiddlewareReturnValue, Request};

fn bench_no_check(c: &mut Criterion) {
    c.bench_function("No check", |bench| {
        bench.iter(|| {
            let v = vec!["a"];

            let _ = v.get(0).unwrap();
        });
    });
}
fn bench_expect(c: &mut Criterion) {
    c.bench_function("Expect", |bench| {
        bench.iter(|| {
            let v = vec!["a"];

            let _ = v.get(0).expect("Yikes");
        });
    });
}
fn bench_assert(c: &mut Criterion) {
    c.bench_function("Assert", |bench| {
        bench.iter(|| {
            let v = vec!["a"];

            let a = v.get(0);
            assert!(a.is_some());
            let _ = a.unwrap();
        });
    });
}
fn bench_remove(c: &mut Criterion) {
    c.bench_function("Remove", |bench| {
        bench.iter(|| {
            let mut v = vec!["a"];

            let _a = v.remove(0);
        });
    });
}

fn bench_route_match(c: &mut Criterion) {
    c.bench_function("Route match", |bench| {
        let mut app = App::<Request, BasicContext>::new_basic();

        fn test_fn_1(
            mut context: BasicContext,
            _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext> + Send,
        ) -> MiddlewareReturnValue<BasicContext> {
            context.body("world");
            Box::new(future::ok(context))
        };

        app.get("/test/hello", middleware![BasicContext => test_fn_1]);

        bench.iter(|| {
            let mut bytes = BytesMut::with_capacity(47);
            bytes.put(&b"GET /test/hello HTTP/1.1\nHost: localhost:8080\n\n"[..]);
            let request = decode(&mut bytes).unwrap().unwrap();
            let matched = app.resolve_from_method_and_path(request.method(), request.path());
            let _response = app.resolve(request, matched).wait().unwrap();
        });
    });
}

fn optimized_bench_route_match(c: &mut Criterion) {
    c.bench_function("Optimized route match", |bench| {
        let mut app = App::<Request, BasicContext>::new_basic();

        fn test_fn_1(
            mut context: BasicContext,
            _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext> + Send,
        ) -> MiddlewareReturnValue<BasicContext> {
            context.body("world");
            Box::new(future::ok(context))
        };

        app.get("/test/hello", middleware![BasicContext => test_fn_1]);
        app._route_parser.optimize();

        bench.iter(|| {
            let mut bytes = BytesMut::with_capacity(47);
            bytes.put(&b"GET /test/hello HTTP/1.1\nHost: localhost:8080\n\n"[..]);
            let request = decode(&mut bytes).unwrap().unwrap();
            let matched = app.resolve_from_method_and_path(request.method(), request.path());
            let _response = app.resolve(request, matched).wait().unwrap();
        });
    });
}

fn bench_route_match_with_param(c: &mut Criterion) {
    c.bench_function("Route match with route params", |bench| {
        let mut app = App::<Request, BasicContext>::new_basic();

        fn test_fn_1(
            mut context: BasicContext,
            _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext> + Send,
        ) -> MiddlewareReturnValue<BasicContext> {
            let body = &context.params.get("hello").unwrap().clone();
            context.body(body);
            Box::new(future::ok(context))
        };

        app.get("/test/:hello", middleware![BasicContext => test_fn_1]);

        bench.iter(|| {
            let mut bytes = BytesMut::with_capacity(48);
            bytes.put(&b"GET /test/world HTTP/1.1\nHost: localhost:8080\n\n"[..]);
            let request = decode(&mut bytes).unwrap().unwrap();
            let matched = app.resolve_from_method_and_path(request.method(), request.path());
            let _response = app.resolve(request, matched).wait().unwrap();
        });
    });
}

fn bench_route_match_with_query_param(c: &mut Criterion) {
    c.bench_function("Route match with query params", |bench| {
        let mut app = App::<Request, BasicContext>::new_basic();

        fn test_fn_1(
            mut context: BasicContext,
            _next: impl Fn(BasicContext) -> MiddlewareReturnValue<BasicContext> + Send,
        ) -> MiddlewareReturnValue<BasicContext> {
            let body = &context.query_params.get("hello").unwrap().clone();
            context.body(body);
            Box::new(future::ok(context))
        };

        app.use_middleware("/", middleware![BasicContext => query_params]);
        app.get("/test", middleware![BasicContext => test_fn_1]);

        bench.iter(|| {
            let mut bytes = BytesMut::with_capacity(54);
            bytes.put(&b"GET /test?hello=world HTTP/1.1\nHost: localhost:8080\n\n"[..]);
            let request = decode(&mut bytes).unwrap().unwrap();
            let matched = app.resolve_from_method_and_path(request.method(), request.path());
            let _response = app.resolve(request, matched).wait().unwrap();
        });
    });
}

criterion_group!(
    benches,
    // bench_no_check,
    // bench_assert,
    // bench_expect,
    // bench_remove,
    optimized_bench_route_match,
    bench_route_match,
    bench_route_match_with_param,
    bench_route_match_with_query_param
);
criterion_main!(benches);
