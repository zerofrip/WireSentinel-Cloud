#![recursion_limit = "512"]

pub mod error;
pub mod middleware;
pub mod openapi;
pub mod routes;

pub use routes::{build_router, AppState};
