//! HTTP access logging middleware: trace id + request parameter logs.

mod middleware;

pub use middleware::logging_middleware;