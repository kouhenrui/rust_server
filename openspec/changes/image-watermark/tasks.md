# Tasks: image-watermark

## 1. Watermark module

- [x] 1.1 Create `src/proc/watermark.rs` with `apply` and the
  two helpers `draw_text_watermark` and `draw_image_watermark`.
- [x] 1.2 `apply` dispatches on `WatermarkSpec` variant
  (`Text` or `Image`).
- [x] 1.3 `draw_text_watermark` reads the configured font via
  `state.fonts.get(...)`, returns `AppError::WatermarkFontMissing`
  when no font path is configured, and
  `AppError::Internal("font not readable: ...")` when the file
  is missing or unreadable.
- [x] 1.4 `draw_image_watermark` resolves the overlay path against
  `state.config.local_source_root`, reads the file, decodes it via
  `source::decode`, and overlays onto the destination.

## 2. Lazy font loading

- [x] 2.1 `state::FontCache` is a `OnceCell<Option<Vec<u8>>>` that
  reads the configured font file on the first call to `get(path)`
  and caches the result. Subsequent calls reuse the cache.
- [x] 2.2 The service starts cleanly with `THUMBOR_WATERMARK_FONT`
  unset or pointing to a non-existent file; the first text-watermark
  request returns 502 `watermark_font_missing`, but startup is
  unaffected.

## 3. Wire it into the handler

- [x] 3.1 `controller/img.rs` calls `watermark::apply` after
  `params.filters.clone().apply(...)` and before the encoder.
  Both `img_get` and `img_post` go through `process_image`, which
  contains this ordering.

## 4. Verification

- [x] 4.1 `cargo check --all-targets` exits 0.
- [x] 4.2 The `http-api` change's integration tests cover the
  no-watermark happy path; watermark paths are exercised manually
  (the spec's "missing font" and "missing overlay" scenarios are
  covered by AppError variants, but end-to-end watermark rendering
  is left for a future change).
