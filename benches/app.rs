extern crate bytes;
extern crate thruster;
extern crate futures;

#[macro_use] extern crate criterion;

use std::boxed::Box;
use futures::{future, Future};
use std::collections::HashMap;
use bytes::{BytesMut, BufMut};

use criterion::Criterion;
use thruster::{decode, App, BasicContext, MiddlewareChain, MiddlewareReturnValue};

fn bench_route_match(c: &mut Criterion) {
  c.bench_function("Route match", |bench| {
    let mut app = App::<BasicContext>::new();

    fn test_fn_1(context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> MiddlewareReturnValue<BasicContext> {
      MiddlewareReturnValue::TSync(BasicContext {
        body: "world".to_owned(),
        params: HashMap::new(),
        query_params: context.query_params
      })
    };

    app.get("/test/hello", vec![test_fn_1]);

    bench.iter(|| {
      let mut bytes = BytesMut::with_capacity(47);
      bytes.put(&b"GET /test/hello HTTP/1.1\nHost: localhost:8080\n\n"[..]);
      let request = decode(&mut bytes).unwrap().unwrap();
      let _response = app.resolve(request).wait().unwrap();
    });
  });
}

fn bench_route_match_with_param(c: &mut Criterion) {
  c.bench_function("Route match with route params", |bench| {
    let mut app = App::<BasicContext>::new();

    fn test_fn_1(context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> MiddlewareReturnValue<BasicContext> {
      MiddlewareReturnValue::TSync(BasicContext {
        body: context.params.get("hello").unwrap().to_owned(),
        params: HashMap::new(),
        query_params: context.query_params
      })
    };

    app.get("/test/:hello", vec![test_fn_1]);

    bench.iter(|| {
      let mut bytes = BytesMut::with_capacity(48);
      bytes.put(&b"GET /test/world HTTP/1.1\nHost: localhost:8080\n\n"[..]);
      let request = decode(&mut bytes).unwrap().unwrap();
      let _response = app.resolve(request).wait().unwrap();
    });
  });
}

fn bench_route_match_with_query_param(c: &mut Criterion) {
  c.bench_function("Route match with query params", |bench| {
    let mut app = App::<BasicContext>::new();

    fn test_fn_1(context: BasicContext, _chain: &MiddlewareChain<BasicContext>) -> MiddlewareReturnValue<BasicContext> {
      MiddlewareReturnValue::TSync(BasicContext {
        body: context.query_params.get("hello").unwrap().to_owned(),
        params: HashMap::new(),
        query_params: context.query_params
      })
    };

    app.get("/test", vec![test_fn_1]);

    bench.iter(|| {
      let mut bytes = BytesMut::with_capacity(54);
      bytes.put(&b"GET /test?hello=world HTTP/1.1\nHost: localhost:8080\n\n"[..]);
      let request = decode(&mut bytes).unwrap().unwrap();
      let _response = app.resolve(request).wait().unwrap();
    });
  });
}

criterion_group!(benches, bench_route_match, bench_route_match_with_param, bench_route_match_with_query_param);
criterion_main!(benches);

