# Design: image-transform

## Context

This change owns the geometric-transform step of the processing
pipeline (`load_source → decode → transform → filter → watermark →
encode`). The transform step is the only one that takes a target
`(w, h)` from the user.

## Goals

- Crop is always applied before resize, because cropping first
  reduces the pixel count that the resize has to touch.
- Three `FitMode` variants — `Cover`, `Contain`, `Stretch` — cover
  the common cases without a special API.
- A single `w` or `h` produces the other dimension for free (aspect
  ratio preserved).

## Non-goals (recap)

No smart crop, no rotation, no multi-crop. See `proposal.md`.

## Decisions

### Crop-then-resize, not resize-then-crop

`transform::apply` runs `crop_to` before `resize_with_fit`.

**Why:** the order is a constant-factor optimization. A 4000×3000
source cropped to 1000×1000 then resized to 200×200 is 1M pixel-
ops; the reverse is 12M. The user gets the same image either way.
Trade-off: a user who *wants* "resize first to make the image small
on disk, then crop the small image" can still do that by running two
requests.

### Out-of-bounds crop returns 400, not 422

`crop_to` checks the crop origin against the source dimensions and
returns `AppError::BadRequest` (→ 400). A crop that lands outside
the image is a caller error, not a server-side data problem, so the
4xx class is correct. (A *malformed* crop string parses to 400 from
`params.rs`; a *valid string* that happens to be out of range is
also 400 — same status, consistent from the caller's perspective.)

### `cover` re-crops inside the resize

`image::DynamicImage::resize_to_fill` performs the cover-fit resize
in one step: it scales to cover the target and crops the overshoot
in the center. The `CropRect` from the user has already been applied
*before* this point, so the user is not in control of where the
inner cover-crop lands. We accept that — the alternative (resizing
inside the user's `CropRect` exactly) is rarely what callers want.

### `contain` letterboxes onto a transparent canvas

`FitMode::Contain` resizes so the source fits *inside* the target,
then `overlay`s the scaled image onto a fresh `DynamicImage::new_rgba8(tw, th)`.
The background is transparent (RGBA, all zeros), which is the only
sane default for an output format that supports alpha (PNG, WebP).
JPEG output of a `contain` image is undefined by JPEG itself; we
still output the transparent canvas and let the JPEG encoder flatten
it (currently: black background — see `image::codecs::jpeg`).

### `Lanczos3` is the resize filter

`resize_to_fill`, `resize_exact`, and `resize` all use
`FilterType::Lanczos3`. It's the highest-quality resampling filter
in `image 0.25`. Trade-off: roughly 2-3x the CPU cost of the
default `Triangle` filter. For a server that runs on dedicated
hardware, this is the right knob to turn.

### Single-dimension requests pass `(0, other)` to `resize_with_fit`

`resize_to_fill(tw, 0, _)` panics. So a request with only `w=200`
is passed as `target = Some((200, 0))`, and `resize_with_fit`
short-circuits when `th == 0` — but in this change we don't have
that path yet; the handler currently rejects `h=0` in `ImgParams::parse`.
The spec still requires it to "compute the height to preserve the
source aspect ratio" so the code is on the to-do list; the test
suite has a test exercising `transform::apply` with `target = (w, 0)`
to drive the implementation.

**Why "TODO":** kept open so this delta's review doesn't have to
land both the spec and the implementation in lock-step. The change
is small enough to be worth landing without a follow-up.

## File map

- `src/proc/transform.rs` — `apply`, `crop_to`, `resize_with_fit`,
  `FitMode`. Added in this change.

## Open questions

- Should the unspecified-dimension computation be done in
  `transform::apply` or in `ImgParams::parse`? Currently planned for
  `transform::apply` because that's where the source image is in scope.
