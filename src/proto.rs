//! Generated protobuf types for the `POST /img` wire format.
//!
//! The actual Rust code is produced at build time by `build.rs` from
//! `proto/api.proto` and dropped into `OUT_DIR`. The output filename
//! follows `<package_name>.<proto_basename>.rs`, so the `package
//! thumbor.v1;` line in `api.proto` makes the file `thumbor.v1.rs`.
//! We re-export it under `api` so the rest of the crate (handlers,
//! tests) can refer to the messages without depending on a hardcoded
//! path.

pub mod api {
    include!(concat!(env!("OUT_DIR"), "/thumbor.v1.rs"));
}
