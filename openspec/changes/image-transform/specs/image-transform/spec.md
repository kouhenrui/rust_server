# Image Transform (Crop & Resize) (delta)

## Purpose

Define the geometric transforms applied to a source image before filters and
watermarks.

## ADDED Requirements

### Requirement: Pipeline order
The service MUST apply an explicit crop before resizing.

#### Scenario: order is crop then resize
- GIVEN a request with both `crop` and `w`/`h`
- WHEN the handler runs the transform pipeline
- THEN the crop is applied first and the resize operates on the cropped image

### Requirement: Explicit crop
The service MUST accept a `crop` parameter of the form `x,y,w,h` in source
pixel coordinates and crop the source to that rectangle.

#### Scenario: crop out of bounds
- GIVEN a crop origin that lies outside the source image
- WHEN the handler applies the crop
- THEN the response status is 400 and the error code is `bad_request`

#### Scenario: valid crop
- GIVEN a `crop` rectangle that lies within the source image
- WHEN the handler applies the crop
- THEN the resulting image has the cropped dimensions

### Requirement: Fit mode `cover`
When `fit=cover`, the service MUST scale and crop the source so that the
result exactly fills the requested `(w, h)` while preserving the source
aspect ratio.

#### Scenario: cover fills exactly
- GIVEN `fit=cover` and `w=200&h=200`
- WHEN the handler runs the transform
- THEN the output image is exactly 200x200 pixels

### Requirement: Fit mode `contain`
When `fit=contain`, the service MUST scale the source so that it fits entirely
inside `(w, h)` while preserving the aspect ratio, and place the scaled image
on a transparent canvas of exactly `(w, h)`.

#### Scenario: contain letterboxes
- GIVEN `fit=contain` and `w=200&h=200`
- WHEN the handler runs the transform
- THEN the output image is exactly 200x200 pixels and the inner image is
  aspect-correct

### Requirement: Fit mode `stretch`
When `fit=stretch`, the service MUST scale the source to exactly `(w, h)`
without preserving the aspect ratio.

#### Scenario: stretch to exact dims
- GIVEN `fit=stretch` and `w=200&h=200`
- WHEN the handler runs the transform
- THEN the output image is exactly 200x200 pixels regardless of source aspect

### Requirement: Default fit
When `fit` is omitted, the service MUST behave as if `fit=cover` was given.

#### Scenario: default fit is cover
- GIVEN a request with `w=200&h=200` and no `fit` parameter
- WHEN the handler runs the transform
- THEN the output is 200x200 with cover behavior (crop-to-fill)

### Requirement: Missing dimensions
The service MUST accept requests that specify only `w` or only `h`; in that
case the unspecified dimension is computed to preserve the source aspect ratio.

#### Scenario: only width given
- GIVEN `w=200` and no `h`
- WHEN the handler runs the transform
- THEN the output width is 200 and the height is computed to preserve the
  source aspect ratio
