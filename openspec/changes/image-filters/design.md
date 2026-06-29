# Design: image-filters

## Context

This change owns the filter-chain step of the processing pipeline
(`load_source → decode → transform → filter → watermark → encode`).
Filters modify pixel values; they do not change geometry.

## Goals

- A simple, readable chain syntax — colon-separated, no quoting
  needed for the common case.
- Validation that fails fast (at parse time, not at apply time) so
  a typo in `filters=` is a 400, not a 500.
- Each filter in the chain is independent of the others; filters
  can be combined in any order.

## Non-goals (recap)

No custom filters, no color-space conversions, no per-filter
side-toggles. See `proposal.md`.

## Decisions

### Colon-separated chain, no JSON

The chain syntax is `grayscale:brightness(20):blur(2.0)`. We
deliberately did not pick JSON (`[{"name":"grayscale"},{"name":"brightness","args":[20]}]`)
or an explicit `&filters=`-per-filter (5 query parameters to spell
5 filters).

**Why:** the chain is the *common* shape — most users will pass
1-3 filters. A colon-separated list is the easiest to read in a URL,
and avoids URL-encoding nested quotes. Trade-off: a filter argument
that needs a comma (currently none) would break the syntax; we
constrain ourselves to single-token arguments for now.

### Parse fails the request, not the chain

A bad filter name, a bad arg count, or a bad arg value rejects the
*whole request* with status 400 and `error.code = "invalid_filter"`.
We do not "skip the bad filter and apply the rest".

**Why:** silent skipping is hard to debug ("why didn't my
`sepia` filter apply? oh, typo — but the request succeeded with
`grayscale:brightness(20)` only?") and breaks the principle of least
surprise. Failing loud is the right default for a request-shape
error.

### Empty chain is the no-op default

`parse("")` returns an empty `FilterChain`, and `FilterChain::apply`
on an empty chain is a no-op. `parse(":grayscale:")` (extra
colons) also returns a single-filter chain, because we `split(':')`
on the colon, `trim` each token, and `filter` out empty tokens.

**Why:** the spec's "empty chain" scenario is "no filters applied",
which is the desired behavior when the caller doesn't pass
`filters=` at all. Tolerating extra colons is just defensive
parsing — frontend code that builds a list and joins with `:` can
end up with `grayscale::blur(2.0)` without us 500-ing on it.

### Filters operate on RGBA, not the source color type

`Filter::apply` for `Brightness` and `Contrast` calls
`img.to_rgba8()` first, runs the in-place mutator, and wraps the
result back as a `DynamicImage::ImageRgba8`.

**Why:** `image 0.25`'s `brighten_in_place` and `contrast_in_place`
are defined on `ImageBuffer<Rgba<u8>, Vec<u8>>` (RGBA). The
`DynamicImage` API has no in-place version of these operations, so
the conversion is the cleanest way to call the in-place mutator.
Trade-off: a `grayscale` source goes through RGB→RGBA and stays
RGBA for the rest of the pipeline. That's fine — the encoder at
the end of the pipeline writes whatever color type the buffer has.

### `blur(sigma)` clamps `sigma` to `0.0..=100.0`

`sigma > 100` is rejected at parse time as `invalid_filter`. The
filter is also rejected for `sigma < 0` (parses to NaN-edge cases)
implicitly because `parse::<f32>` accepts the literal but the
range check catches the bad case.

**Why 100:** at sigma 100 the whole image blurs to a single average
color; the result is functionally useless. The cap rejects obvious
typos (a misplaced decimal, an extra zero) without the user having
to know our exact tolerance.

## File map

- `src/proc/filter.rs` — `Filter`, `FilterChain`, `parse_one`,
  `exactly_one`, unit tests. Added in this change.

## Open questions

- Should we add a `noop` filter explicitly so users can write
  `?filters=sepia:noop` and get a parse error on the first? No,
  keeping the unknown-filter error visible is more useful.
