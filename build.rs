// Compile proto/api.proto → generated Rust types in OUT_DIR.
//
// Why this file: `prost-build` runs at `cargo build` time and emits a
// Rust file under `OUT_DIR` named `<package_name>.<proto_basename>.rs`.
// With `package thumbor.v1;` in api.proto, that's `thumbor.v1.rs`,
// which `src/proto.rs` pulls in via `include!`. Doing the codegen as a
// `build.rs` keeps the generated code out of version control and the
// `cargo check` loop stays single-step (no separate code-generation
// step to remember).
//
// Why `bytes(["."])`: the `bytes` field in `ImageResponse` holds the
// encoded image body. Calling `.bytes(["."])` makes prost emit that
// field as `bytes::Bytes` instead of the default `Vec<u8>` — we want
// the `Bytes` so we can move it into the axum response body without
// an extra copy. The argument `["."]` means "all `bytes` fields in
// all protos we compile"; today there's only one, but the call site
// is forward-compatible if more `bytes` fields get added.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // `protoc_bin_vendored::protoc_bin_path()` returns the path to a
    // precompiled `protoc` binary that's been downloaded and extracted
    // by this build-dep on first run. Passing it explicitly to
    // prost-build means we never call out to whatever the user happens
    // to have on PATH (or nothing on Windows).
    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    let mut config = prost_build::Config::new();
    config.protoc_executable(protoc);
    config.bytes(["."]);
    config.compile_protos(&["proto/api.proto"], &["proto/"])?;
    println!("cargo:rerun-if-changed=proto/api.proto");
    Ok(())
}
