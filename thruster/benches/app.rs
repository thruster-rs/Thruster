#[macro_use]
extern crate criterion;

use bytes::{BufMut, BytesMut};
use std::boxed::Box;
use tokio;

use criterion::Criterion;
use thruster::middleware::query_params::query_params;
use thruster::{async_middleware, middleware_fn, decode, App, BasicContext, MiddlewareNext, MiddlewareResult, MiddlewareReturnValue, Request};

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

#[middleware_fn]
async fn test_fn_1(
    mut context: BasicContext,
    _next: MiddlewareNext<BasicContext>,
) -> MiddlewareResult<BasicContext> {
    context.body("world");
    Ok(context)
}

fn bench_route_match(c: &mut Criterion) {
    c.bench_function("Route match", |bench| {
        let mut app = App::<Request, BasicContext>::new_basic();

        app.get("/test/hello", async_middleware!(BasicContext, [test_fn_1]));

        let mut rt = tokio::runtime::Runtime::new().unwrap();
        bench.iter(|| {
            let mut bytes = BytesMut::with_capacity(47);
            bytes.put(&b"GET /test/hello HTTP/1.1\nHost: localhost:8080\n\n"[..]);
            let request = decode(&mut bytes).unwrap().unwrap();
            let matched = app.resolve_from_method_and_path(request.method(), request.path());
            rt.block_on(async {
                let _response = app.resolve(request, matched).await.unwrap();
            });
        });
    });
}

fn optimized_bench_route_match(c: &mut Criterion) {
    c.bench_function("Optimized route match", |bench| {
        let mut app = App::<Request, BasicContext>::new_basic();

        app.get("/test/hello", async_middleware!(BasicContext, [test_fn_1]));
        app._route_parser.optimize();

        let mut rt = tokio::runtime::Runtime::new().unwrap();
        bench.iter(|| {
            let mut bytes = BytesMut::with_capacity(47);
            bytes.put(&b"GET /test/hello HTTP/1.1\nHost: localhost:8080\n\n"[..]);
            let request = decode(&mut bytes).unwrap().unwrap();
            let matched = app.resolve_from_method_and_path(request.method(), request.path());
            rt.block_on(async {
                let _response = app.resolve(request, matched).await.unwrap();
            });
        });
    });
}

#[middleware_fn]
async fn test_params_1(
    mut context: BasicContext,
    _next: MiddlewareNext<BasicContext>,
) -> MiddlewareResult<BasicContext> {
    let body = &context.params.get("hello").unwrap().clone();
    context.body(body);
    Ok(context)
}

fn bench_route_match_with_param(c: &mut Criterion) {
    c.bench_function("Route match with route params", |bench| {
        let mut app = App::<Request, BasicContext>::new_basic();

        app.get("/test/:hello", async_middleware!(BasicContext, [test_params_1]));

        let mut rt = tokio::runtime::Runtime::new().unwrap();
        bench.iter(|| {
            let mut bytes = BytesMut::with_capacity(48);
            bytes.put(&b"GET /test/world HTTP/1.1\nHost: localhost:8080\n\n"[..]);
            let request = decode(&mut bytes).unwrap().unwrap();
            let matched = app.resolve_from_method_and_path(request.method(), request.path());
            rt.block_on(async {
                let _response = app.resolve(request, matched).await.unwrap();
            });
        });
    });
}

#[middleware_fn]
async fn test_query_params_1(
    mut context: BasicContext,
    _next: MiddlewareNext<BasicContext>,
) -> MiddlewareResult<BasicContext> {
    let body = &context.query_params.get("hello").unwrap().clone();
    context.body(body);
    Ok(context)
}
fn bench_route_match_with_query_param(c: &mut Criterion) {
    c.bench_function("Route match with query params", |bench| {
        let mut app = App::<Request, BasicContext>::new_basic();

        app.use_middleware("/", async_middleware!(BasicContext, [query_params]));
        app.get("/test", async_middleware!(BasicContext, [test_query_params_1]));

        let mut rt = tokio::runtime::Runtime::new().unwrap();
        bench.iter(|| {
            let mut bytes = BytesMut::with_capacity(54);
            bytes.put(&b"GET /test?hello=world HTTP/1.1\nHost: localhost:8080\n\n"[..]);
            let request = decode(&mut bytes).unwrap().unwrap();
            let matched = app.resolve_from_method_and_path(request.method(), request.path());
            rt.block_on(async {
                let _response = app.resolve(request, matched).await.unwrap();
            });
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
