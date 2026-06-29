# Change: image-watermark

## Why

Operators need to brand or copyright the images that go through the
service. The watermark step is the last visual modification before
the image is encoded and returned. Two flavors are common — text
(band name, "© 2026 ACME") and image (a logo PNG) — and both should
work without separate code paths.

## What Changes

- Accept a text watermark via `watermark=Hello@10,10`; render the
  text with the TTF pointed to by `THUMBOR_WATERMARK_FONT`, with the
  top-left at pixel `(10, 10)` of the processed image.
- Accept an image watermark via `watermark=image:logo.png@10,10`;
  overlay the file at `logo.png` (resolved against
  `THUMBOR_LOCAL_SOURCE_ROOT`) on top of the processed image at
  pixel `(10, 10)`.
- Reject a text watermark with status 502 and error code
  `watermark_font_missing` when no font is configured.
- Reject a missing overlay file (image watermark) with status 404
  and error code `source_not_found`.
- Load the configured font lazily on the first text-watermark
  request, so a deployment without a font still starts cleanly.

## Capabilities

- `image-watermark`: Text watermark, image watermark, font-required
  semantics, lazy font loading.

## Impact

- **Code:** new `src/proc/watermark.rs` with `apply`,
  `draw_text_watermark`, `draw_image_watermark`. Called by the
  `/img` handler after the filter chain and before the encoder.
- **Dependencies:** `imageproc 0.25`'s `drawing::draw_text_mut` for
  the text path, plus `ab_glyph 0.2` for the font representation
  (`FontVec`). The `ab_glyph` dependency **must** be listed
  explicitly in `[dependencies]` because it is not re-exported by
  `imageproc` (this is documented in `AGENTS.md §6`).
- **Config:** `THUMBOR_WATERMARK_FONT` (already in the
  `runtime-config` change) and `THUMBOR_LOCAL_SOURCE_ROOT` (image
  watermarks resolve relative to the local source root, same as
  regular sources).
- **Failure modes:** the service starts cleanly with no font
  configured; the first text-watermark request gets a 502.
  Operators that don't want text watermarks at all can simply
  reject such requests upstream.

## Non-goals

- Text *styling* via the URL (font size, color, weight). The
  current implementation hardcodes a 32px white text color. A
  future change can add `font_size`, `color`, etc. via additional
  query parameters or a richer watermark grammar.
- Animated GIF watermarks (the source is a single still frame).
- Watermark positioning hints like "center" or "bottom-right".
  Coordinates are always absolute pixels with the top-left of the
  watermark at `(x, y)`. Negative coordinates are allowed and
  produce a partially off-canvas watermark.
- Watermark caching (the font bytes are cached in `FontCache`, but
  the rendered overlay is recomputed per request — the overlay
  depends on the post-transform image, which is per-request).

## Affected domains

- **image-watermark** (created) — the only spec modified or
  created by this change.
