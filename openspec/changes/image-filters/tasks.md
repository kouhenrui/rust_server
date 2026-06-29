# Tasks: image-filters

## 1. Filter module

- [x] 1.1 Create `src/proc/filter.rs` with the `Filter` enum
  (`Grayscale`, `Brightness(i32)`, `Contrast(i32)`, `Blur(f32)`)
  and the `FilterChain` newtype.
- [x] 1.2 `Filter::apply` dispatches on the variant; for
  `Brightness` and `Contrast` it converts to `ImageRgba8` first so
  `brighten_in_place` / `contrast_in_place` are available.
- [x] 1.3 `FilterChain::parse` `split(':')`s the input, `trim`s each
  token, drops empty tokens, and parses each remaining token via
  `parse_one`.
- [x] 1.4 `parse_one` finds the first `(`, splits name and args
  (with `None` for filter names that take no args), then dispatches
  on the name. Unknown names return
  `AppError::Filter("unknown filter '...'")` which surfaces as 400
  `invalid_filter`.
- [x] 1.5 `exactly_one` enforces "1 argument" for the filters that
  take one. (All current built-ins do; this is a placeholder for
  the multi-arg case.)

## 2. Range validation

- [x] 2.1 `blur(sigma)` rejects `sigma < 0` or `sigma > 100` at parse
  time with `AppError::Filter("blur sigma N out of range 0..=100")`.

## 3. Wire it into the handler

- [x] 3.1 `/img` handler calls `params.filters.clone().apply(&mut img)`
  after `transform::apply` and before `watermark::apply`.

## 4. Tests

- [x] 4.1 Unit test `parses_full_chain` — three filters parse in
  order.
- [x] 4.2 Unit test `rejects_unknown_filter` — `sepia` returns
  `AppError::Filter`.
- [x] 4.3 Unit test `rejects_bad_arg_count` —
  `brightness(1,2)` returns error.
- [x] 4.4 Unit test `rejects_out_of_range_blur` —
  `blur(500)` returns error.
- [x] 4.5 Unit test `empty_string_yields_empty_chain` — `""`
  parses to an empty chain, no error.

## 5. Verification

- [x] 5.1 `cargo check --all-targets` exits 0.
- [x] 5.2 `cargo test --lib` passes (5 unit tests above).
