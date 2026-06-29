# Change: image-transform

## Why

Most image-processing requests are about getting the right *geometry*:
crop to a region, then resize to a target box. The order, the fit
modes, and the defaults for missing parameters are the contract
callers depend on, and they affect every request that touches the
service.

## What Changes

- Apply an explicit `crop=x,y,w,h` to the source image **before**
  resizing.
- Accept three `fit` modes — `cover` (default), `contain`,
  `stretch` — to control how the source aspect ratio interacts with
  the target box.
- Default to `fit=cover` when `fit` is omitted.
- Accept requests that specify only `w` or only `h`; the unspecified
  dimension is computed to preserve the source aspect ratio.
- Reject out-of-bounds crop origins with status 400 and error code
  `bad_request`.

## Capabilities

- `image-transform`: Pipeline order, explicit crop, three fit modes,
  default fit, single-dimension requests.

## Impact

- **Code:** new `src/proc/transform.rs` with `apply`, `crop_to`,
  `resize_with_fit`, and the `FitMode` enum. Called by the `/img`
  handler.
- **Dependencies:** `image 0.25`'s `imageops` for `crop_imm`,
  `resize_to_fill`, `resize_exact`, `resize`, and `overlay`.
- **Order:** every `/img` request that passes both `crop` and a
  target size pays the cost of crop-then-resize. Operators that don't
  need a crop can simply omit `crop` and pay nothing extra.

## Non-goals

- Smart content-aware cropping (face detection, saliency maps).
- Sub-region rotation or flipping.
- Multiple crops in one request.
- Returning a "we picked these dimensions" header when only one of
  `w` / `h` is given — the unspecified dimension is computed
  silently, which is the convention the rest of the image-processing
  ecosystem follows.

## Affected domains

- **image-transform** (created) — the only spec modified or created
  by this change.
