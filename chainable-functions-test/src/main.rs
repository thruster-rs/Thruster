use std::{future::Future, ops::Add};

type AsyncNext<T, TO> = Box<dyn Fn(T) -> TO + Send + Sync>;
type Next<T, TO> = Box<dyn Fn(T) -> TO>;

#[tokio::main]
async fn main() {
    struct Person {
        first: String,
        last: String,
    }

    let f: ChainableFn<_, _, _> = (
        // Parse as csv
        |a: String, next: AsyncNext<Person, _>| async move {
            let mut split = a.split(",");

            let val: Person = (next)(Person {
                first: split.next().unwrap().to_string(),
                last: split.next().unwrap().to_string(),
            })
            .await;

            format!("{},{}", val.first, val.last)
        },
        |mut val: Person, _next: AsyncNext<Person, _>| async move {
            val.last = "<Redacted>".to_string();

            val
        },
    )
        .into();

    println!("test: {}", (f.f)("Anakin,Skywalker".to_string()).await);

    // Sync triplet
    let f: ChainableFn<_, _, _> = (_a, _b, _c).into();

    println!("test: {}", (f.f)(1));

    // Async closure triplet
    let f: ChainableFn<_, _, _> = (
        |a: i32, next: AsyncNext<i32, _>| async move {
            let val = (next)(a + 1).await;

            val
        },
        |b: i32, next: AsyncNext<String, _>| async move {
            let val: String = (next)(b.to_string()).await;
            let val = val.parse::<i32>().unwrap();

            val
        },
        |c: String, _next: AsyncNext<String, _>| async move { format!("10{c}") },
    )
        .into();

    println!("test: {}", (f.f)(1).await);

    // Async triplet
    let f: ChainableFn<_, _, _> = (_a_async, _b_async, _c_async).into();

    println!("test: {}", (f.f)(1).await);

    let f: ChainableFn<_, _, _> = (_c_async,).into();

    println!("test: {}", (f.f)("test".to_string()).await);
}

fn _a(a: i32, next: Next<i32, i32>) -> i32 {
    (next)(a + 1)
}
fn _b(b: i32, next: Next<String, String>) -> i32 {
    (next)(b.to_string()).parse::<i32>().unwrap()
}
fn _c(c: String, _next: Next<String, String>) -> String {
    format!("10{c}")
}

async fn _a_async<TIO>(a: i32, next: AsyncNext<i32, TIO>) -> i32
where
    TIO: Future<Output = i32> + Send,
{
    let val = (next)(a + 1).await;

    val
}

async fn _b_async<TIO>(b: i32, next: AsyncNext<String, TIO>) -> i32
where
    TIO: Future<Output = String> + Send,
{
    let val = (next)(b.to_string()).await.parse::<i32>().unwrap();

    val
}

async fn _c_async(c: String, _next: AsyncNext<String, String>) -> String {
    format!("10{c}")
}

// chainable_functions::chainable_functions!(A, B, C, D, E, F, G, H, I, J, K, L, M, N);
chainable_functions::chainable_functions!(A, B, C, D);

// Test Stuff
struct Container<A, B> {
    inner: Box<dyn Into<dyn ZCombine<A, B>>>,
}

trait ZCombine<A, B>: Sized {
    fn combine(self, a: A) -> B {
        todo!()
    }
}

impl<A, B, T, TO, InnerType> Into<ChainableFn<T, TO, InnerType>> for Box<dyn ZCombine<A, B>> {
    fn into(self) -> ChainableFn<T, TO, InnerType> {
        todo!()
    }
}

impl<A, B, C, D> ZCombine<(D,), (A, B, C, D)> for (A, B, C) {}

async fn tst() {
    let f: ChainableFn<_, _, _> = (_a_async, _b_async, _c_async).into();

    println!("test: {}", (f.f)(1).await);

    let f: ChainableFn<_, _, _> = (_c_async,).into();

    println!("test: {}", (f.f)("test".to_string()).await);
}
