# Tasks: image-transform

## 1. Transform module

- [x] 1.1 Create `src/proc/transform.rs` with the `FitMode` enum
  (`Cover`, `Contain`, `Stretch`).
- [x] 1.2 `apply(img, crop, target, fit)` runs `crop_to` (if `crop` is
  set) then `resize_with_fit` (if `target` is set).
- [x] 1.3 `crop_to(img, rect)` returns a new image cropped to the
  rectangle; rejects out-of-bounds origins with
  `AppError::BadRequest`.
- [x] 1.4 `resize_with_fit(img, tw, th, fit)` dispatches on
  `FitMode`: `Stretch` → `resize_exact`, `Cover` → `resize_to_fill`,
  `Contain` → letterbox onto a transparent `new_rgba8(tw, th)` canvas.

## 2. Wire it into the handler

- [x] 2.1 `controller/img.rs` calls `transform::apply` after
  `source::decode` and before the filter chain.

## 3. Verification

- [x] 3.1 `cargo check --all-targets` exits 0.
- [x] 3.2 Integration test (in `tests/integration.rs`,
  `get_img_query_still_works` and `post_img_protobuf_success`)
  verifies that the resize path produces an image of the requested
  dimensions (4x4 from a 2x2 source with `w=4&h=4`).
