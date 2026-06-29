# Change: image-filters

## Why

A common request shape is "transform the image, then apply a sequence
of color/blur effects". The filter chain is the per-image step that
applies these effects in a fixed order. The chain syntax, the order,
and the validation of each filter's arguments are the contract
callers depend on.

## What Changes

- Accept a `filters` query parameter whose value is a colon-separated
  list of `name` or `name(arg)` invocations, e.g.
  `grayscale:brightness(20):blur(2.0)`.
- Apply filters in the order they appear in the chain.
- Support at minimum these built-in filters: `grayscale`,
  `brightness(N)`, `contrast(N)`, `blur(sigma)`.
- Reject any filter that has the wrong arg count, an
  unparseable argument, or an out-of-range argument with status 400
  and error code `invalid_filter`.
- Treat an empty `filters` value as "no filters", not as a parse
  error.

## Capabilities

- `image-filters`: Chain syntax, filter ordering, built-in filters,
  argument validation.

## Impact

- **Code:** new `src/proc/filter.rs` with `Filter`, `FilterChain`,
  `parse_one`, `exactly_one`, and the unit tests. Called by the
  `/img` handler after transform and before watermark.
- **Dependencies:** `image 0.25`'s `imageops::colorops` for
  `brighten_in_place` and `contrast_in_place`. `image::DynamicImage`'s
  `grayscale` and `blur` methods are used directly.
- **Pipeline position:** filters run after `transform::apply` and
  before `watermark::apply`. This ordering matters: a `blur` after
  a watermark would smear the watermark; a `blur` before a watermark
  blurs the source but the watermark is drawn on top of the
  already-blurred image.

## Non-goals

- User-defined custom filters (no plugin mechanism).
- Per-filter enable/disable via separate query parameters (the
  filter's *presence* in the chain is the toggle).
- Color-space conversions (sRGB → linear, etc.). All filters operate
  on the image as decoded.
- Side effects on the source image (filters always operate on the
  in-memory `DynamicImage`).

## Affected domains

- **image-filters** (created) — the only spec modified or created
  by this change.
