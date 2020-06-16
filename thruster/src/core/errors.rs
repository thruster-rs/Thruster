use crate::core::context::Context;
use std::error::Error as StdError;

pub struct ThrusterError<C> {
    pub context: C,
    pub message: String,
    pub status: u32,
    pub cause: Option<Box<dyn StdError>>,
}

pub trait Error<C> {
    fn build_context(self) -> C;
}

impl<C: Context> Error<C> for ThrusterError<C> {
    fn build_context(self) -> C {
        self.context
    }
}

pub trait ErrorSet<C> {
    fn parsing_error(context: C, error: &str) -> ThrusterError<C>;
    fn generic_error(context: C) -> ThrusterError<C>;
    fn unauthorized_error(context: C) -> ThrusterError<C>;
    fn not_found_error(context: C) -> ThrusterError<C>;
}

impl<C: Context> ErrorSet<C> for ThrusterError<C> {
    fn parsing_error(context: C, error: &str) -> ThrusterError<C> {
        ThrusterError {
            context,
            message: format!("Failed to parse '{}'", error),
            status: 400,
            cause: None,
        }
    }

    fn generic_error(context: C) -> ThrusterError<C> {
        ThrusterError {
            context,
            message: "Something didn't work!".to_string(),
            status: 400,
            cause: None,
        }
    }

    fn unauthorized_error(context: C) -> ThrusterError<C> {
        ThrusterError {
            context,
            message: "Unauthorized".to_string(),
            status: 401,
            cause: None,
        }
    }

    fn not_found_error(context: C) -> ThrusterError<C> {
        ThrusterError {
            context,
            message: "Not found".to_string(),
            status: 404,
            cause: None,
        }
    }
}

impl<C: Context> std::fmt::Debug for ThrusterError<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThrusterError")
            .field("message", &self.message)
            .field("status", &self.status)
            .finish()
    }
}
