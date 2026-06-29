# Design: image-watermark

## Context

This change owns the watermark step of the processing pipeline
(`load_source → decode → transform → filter → watermark → encode`).
Watermarks are composited onto the already-transformed and
already-filtered image, so a filter like `blur` does not smear the
watermark and the watermark is the *last* visual change before
encoding.

## Goals

- Two flavors: text and image, distinguished by a `image:` prefix.
- Lazy font loading — the service must start even with no font
  configured.
- A missing overlay file is a 404 (`source_not_found`), not a 500
  — the caller asked for an overlay that doesn't exist.

## Non-goals (recap)

No styling knobs, no positioning hints, no animated watermarks. See
`proposal.md`.

## Decisions

### Watermark runs after filters

`handler::img` calls `transform::apply` → `params.filters.apply` →
`watermark::apply` → `encode`. The order matters: if a watermark
were applied *before* the filter chain, a `blur` filter would smear
the watermark into the image; running after keeps the watermark
crisp.

### Text watermark is rendered to a transparent canvas, then overlaid

`draw_text_watermark` computes the text's bounding box (sum of
glyph advances for the width, `ascent - descent` for the height),
creates a `DynamicImage::new_rgba8(text_w, text_h)`, calls
`imageproc::drawing::draw_text_mut` on that canvas, and then
`overlay`s the result onto the main image at the requested `(x, y)`.

**Why a temp canvas:** `imageproc`'s `draw_text_mut` does not
clip out-of-canvas drawing, so drawing on the main image would
either require pre-clipping the text or accepting garbage outside
the text bounds. The temp canvas is exactly text-sized, and
`image::imageops::overlay` handles the off-canvas case for us
(it silently drops pixels outside the destination).

**Trade-off:** the temp canvas allocation is per-request. For a
single-shot watermark (the common case) this is fine. We could
pool canvases but the savings are not worth the API complexity.

### `image:foo.png` prefix is the only syntax-level dispatch

`params::parse_watermark` checks for a leading `image:`; everything
else is treated as a text watermark. Coordinates come from the
`@x,y` suffix.

**Why:** the `:`-prefix scheme keeps the rest of the parser
identical to the text path. The alternative (separate `text` and
`image` query parameters) doubles the number of request
parameters, and the URL gets ugly with optional combinations.

### `rsplit_once('@')` for the coordinate split

A text watermark can contain `@` characters (e.g. an email-style
watermark `support@acme.com@10,10`). `rsplit_once('@')` cuts from
the right so the rightmost `@` is the coordinate separator and
the text can contain anything else.

### Image watermarks resolve against the local source root

`draw_image_watermark` joins the overlay path onto
`THUMBOR_LOCAL_SOURCE_ROOT` (or uses the literal path when the
root is unset). The same convention as `/img?src=` for the main
image — no special root for watermark files.

**Why:** operators configure *one* root and it covers all
file-system lookups. Adding a second root just for watermarks
would be a footgun.

### `FontCache` reads the file once, then reuses

`state::FontCache` is a `OnceCell<Option<Vec<u8>>>` around the
font bytes. `get(path)` reads the file the first time and caches
the `Vec<u8>` (or `None` if the file is missing). The `Bytes`
are then parsed into a `FontVec` on every request — the parse is
fast and avoids the lifetime issues of caching a `FontVec`
behind an `RwLock`.

**Trade-off:** the bytes are re-parsed per request. The parse
itself is a few hundred microseconds; not worth optimizing.

### Hard-coded 32px white text

`draw_text_watermark` uses `PxScale::from(32.0)` and
`Rgba([255, 255, 255, 255])` (white, fully opaque).

**Why hard-coded:** the spec doesn't require styled watermarks;
32px is a reasonable default that reads well on most thumbnail
sizes. Adding a `?font_size=` and `?font_color=` query parameter
is a future change with its own design (the URL grammar gets
messy, and we'd need to keep backward compatibility).

## File map

- `src/proc/watermark.rs` — `apply`, `draw_text_watermark`,
  `draw_image_watermark`. Added in this change.
- `src/state.rs` — `FontCache` (used by `draw_text_watermark`).
  Pre-existing, modified to expose the cache.

## Open questions

- Should the text-rendering path support multi-line text
  (e.g. `Hello\nWorld@10,10`)? Not in this change — the parser
  already takes a single `text` field, and a `\n` in a URL is
  awkward to round-trip.
