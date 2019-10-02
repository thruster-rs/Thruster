#[macro_use]
extern crate templatify;

#[cfg(test)]
#[macro_use] extern crate thruster_core;

pub mod app;

#[cfg(all(feature = "thruster_testing", not(feature = "thruster_async_await")))]
pub mod testing;
#[cfg(all(feature = "thruster_testing", feature = "thruster_async_await"))]
pub mod testing_async;
