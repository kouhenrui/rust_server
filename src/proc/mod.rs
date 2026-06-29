//! Image transformation pipeline. Each module exposes small functions that
//! mutate a `DynamicImage` in place, so handlers can compose them
//! (`crop -> resize -> filters -> watermark`).

pub mod filter;
pub mod transform;
pub mod watermark;
