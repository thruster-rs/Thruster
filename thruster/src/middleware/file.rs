use bytes::Bytes;
use bytes::BytesMut;
use chashmap::CHashMap;
use lazy_static::*;
use thruster_proc::middleware_fn;
use std::env;
use std::fs::File;
use std::io::prelude::*;
use std::iter::FromIterator;

use crate::map_try;
use crate::core::context::Context;
use crate::core::errors::{ErrorSet, ThrusterError as Error};
use crate::core::{MiddlewareNext, MiddlewareResult};

lazy_static! {
    static ref CACHE: CHashMap<String, Vec<u8>> = { CHashMap::new() };
    ///
    /// ROOT_DIR, stored in the RUST_ROOT_DIR env var dictates where
    /// the `file` middleware serves from.
    ///
    static ref ROOT_DIR: String = env::var("RUST_ROOT_DIR").unwrap_or_else(|_| "/".to_string());
    ///
    /// HOST_DIR, stored in the RUST_HOST_DIR env var dictates where
    /// the `file` middleware serves from.
    ///
    static ref HOST_DIR: String = env::var("RUST_HOST_DIR").unwrap_or_else(|_| "".to_string());
}

///
/// Middleware to set send a static file.
///
/// The `file` middleware takes the currently requested route
/// and checks the local filesystem for that file. If found,
/// then it returns the file, if not, then it returns a NotFound
/// error.
///
/// `file`, and the underlying `get_file`, also attempt to cache
/// any files that it reads. The underlying data is stored in a
/// CHashMap. This feature can be turned off by setting the env
/// var RUST_CACHE=off
///
#[middleware_fn(_internal)]
pub async fn file<T: 'static + Context + Send>(
    mut context: T,
    _next: MiddlewareNext<T>,
) -> MiddlewareResult<T> {
    let root_dir: &'static str = &ROOT_DIR;
    let host_dir: &'static str = &HOST_DIR;
    let path = context.route().replace(root_dir, host_dir);
    let content = map_try!(get_file(&path), Err(_) => Error::not_found_error(context));

    context.set_body_bytes(content);
    Ok(context)
}

///
/// An access point to the underlying cached file implementation
/// in case the developer wants to write custom parsing for the
/// path.
///
/// `get_file` attempts to cache any files that it reads. The
/// underlying data is stored in a CHashMap. This feature can
/// be turned off by setting the env var RUST_CACHE=off
///
pub fn get_file(path: &str) -> Result<Bytes, std::io::Error> {
    let path = path.replace("..", "");

    let is_cache_off = env::var("RUST_CACHE").unwrap_or_else(|_| "off".to_string()) == "off";

    let fetched = CACHE.get(&path);
    match (is_cache_off, fetched) {
        (false, Some(val)) => Ok(Bytes::from(BytesMut::from_iter(&*val))),
        (false, None) => {
            let val = read_file(&path)?;
            CACHE.insert(path.clone(), val);
            let val = CACHE.get(&path).unwrap();
            Ok(Bytes::from(BytesMut::from_iter(&*val)))
        }
        (true, _) => {
            let val = read_file(&path)?;
            Ok(Bytes::from(BytesMut::from_iter(&*val)))
        }
    }
}

fn read_file(path: &str) -> Result<Vec<u8>, std::io::Error> {
    let mut file = File::open(path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)?;

    Ok(contents)
}
