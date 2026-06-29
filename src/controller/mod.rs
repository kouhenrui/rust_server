//! HTTP controllers. Wire-format translation happens here; the pixel
//! pipeline is shared via [`img::process_image`], responses via
//! [`crate::response::ImageOutcome`].

pub mod health;
pub mod img;
