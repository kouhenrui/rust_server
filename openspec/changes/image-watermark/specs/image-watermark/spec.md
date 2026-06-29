# Image Watermark (delta)

## Purpose

Define text and image overlays composited onto the source image as the last
processing step before encoding.

## ADDED Requirements

### Requirement: Text watermark
The service MUST accept a `watermark` value of the form `Hello@10,10` to draw
the literal text `Hello` with its top-left at pixel `(10, 10)` of the
processed image.

#### Scenario: text watermark applied
- GIVEN `THUMBOR_WATERMARK_FONT` is set to a readable TTF
- AND `watermark=Hello@10,10`
- WHEN the handler applies the watermark
- THEN the response is a successful image and the text "Hello" appears at
  pixel (10, 10)

### Requirement: Image watermark
The service MUST accept a `watermark` value of the form `image:logo.png@10,10`
to overlay the image at `logo.png` (resolved against the local source root)
with its top-left at pixel `(10, 10)`.

#### Scenario: image watermark applied
- GIVEN `watermark=image:logo.png@10,10` and the overlay file exists
- WHEN the handler applies the watermark
- THEN the response is a successful image containing the overlay at (10, 10)

#### Scenario: missing overlay file
- GIVEN `watermark=image:missing.png@0,0`
- WHEN the handler loads the overlay
- THEN the response status is 404 and the error code is `source_not_found`

### Requirement: Font is required for text watermarks
The service MUST reject a text watermark with status 502 and error code
`watermark_font_missing` when no font is configured.

#### Scenario: text watermark without font
- GIVEN `THUMBOR_WATERMARK_FONT` is unset
- AND a request with `watermark=Hello@0,0`
- WHEN the handler applies the watermark
- THEN the response status is 502 and the error code is `watermark_font_missing`

### Requirement: Font is loaded lazily
The service MUST NOT require the configured font file to exist at startup; it
is loaded on the first text-watermark request.

#### Scenario: service starts without font
- GIVEN `THUMBOR_WATERMARK_FONT` points to a non-existent file
- WHEN the service starts
- THEN startup succeeds and no error is logged for the missing font
