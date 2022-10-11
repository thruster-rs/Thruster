use crate::core::context::Context;
use std::error::Error as StdError;

pub struct ThrusterError<C> {
    pub context: C,
    pub message: String,
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
    fn parsing_error(mut context: C, error: &str) -> ThrusterError<C> {
        context.status(400);

        ThrusterError {
            context,
            message: format!("Failed to parse '{}'", error),
            cause: None,
        }
    }

    fn generic_error(mut context: C) -> ThrusterError<C> {
        context.status(400);

        ThrusterError {
            context,
            message: "Something didn't work!".to_string(),
            cause: None,
        }
    }

    fn unauthorized_error(mut context: C) -> ThrusterError<C> {
        context.status(401);

        ThrusterError {
            context,
            message: "Unauthorized".to_string(),
            cause: None,
        }
    }

    fn not_found_error(mut context: C) -> ThrusterError<C> {
        context.status(404);

        ThrusterError {
            context,
            message: "Not found".to_string(),
            cause: None,
        }
    }
}

impl<C> std::fmt::Debug for ThrusterError<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThrusterError")
            .field("message", &self.message)
            .finish()
    }
}

impl<C: Clone> Clone for ThrusterError<C> {
    fn clone(&self) -> Self {
        ThrusterError {
            context: self.context.clone(),
            message: self.message.clone(),
            cause: None,
        }
    }
}

impl<C: Clone + Default> From<Box<dyn std::error::Error>> for ThrusterError<C> {
    fn from(e: Box<dyn std::error::Error>) -> Self {
        ThrusterError {
            context: C::default(),
            message: e.to_string(),
            cause: Some(e),
        }
    }
}
