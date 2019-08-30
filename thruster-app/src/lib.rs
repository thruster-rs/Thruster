#[macro_use]
extern crate templatify;

#[cfg(test)]
#[macro_use] extern crate thruster_core;

pub mod app;

#[cfg(feature = "thruster_testing")]
pub mod testing;
