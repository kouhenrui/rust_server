# Image Processing Pipeline

## Purpose

Define the end-to-end orchestration from validated parameters to encoded output
bytes, shared by `GET /img` and `POST /img`.

## Requirements

### Requirement: Single orchestration entry

`controller/img::process_image` MUST be the sole orchestration function for
image transformation. Both HTTP handlers MUST call it after parameter parsing.

#### Scenario: handler delegation

- GIVEN valid `ImgParams`
- WHEN `img_get` or `img_post` runs
- THEN `process_image(state, params)` is invoked exactly once

### Requirement: Pipeline ordering

`process_image` MUST execute steps in this order:

1. `source::load_source` — obtain raw bytes
2. `source::decode` — decode to `DynamicImage`
3. `proc::transform::apply` — optional crop, then resize/fit
4. `FilterChain::apply` — filters in declaration order
5. `proc::watermark::apply` — optional text or image overlay
6. `encode` — PNG / JPEG / WebP output

#### Scenario: crop before resize

- GIVEN both `crop` and `w`/`h` are set
- WHEN the pipeline runs
- THEN cropping happens before resizing

### Requirement: Output encoding

The `encode` function in `controller/img.rs` MUST support `png` (default),
`jpeg` (quality 85), and `webp` (lossless). Success responses MUST carry the
matching `content_type` MIME string.

#### Scenario: jpeg output

- GIVEN `format=jpeg` and a successful pipeline
- WHEN the response is built
- THEN `data.content_type` is `image/jpeg` and bytes are valid JPEG

### Requirement: Format sniffing

Source format detection MUST use `AppState::sniff_format` (`image::guess_format`)
on raw bytes, not URL extensions or `Content-Type` headers.

### Requirement: Lazy font cache

`AppState.fonts` (`FontCache`) MUST lazily load the watermark TTF on first use
via `OnceCell`, so startup succeeds when no font is configured.

#### Scenario: font not required at startup

- GIVEN `THUMBOR_WATERMARK_FONT` is unset
- WHEN the service starts
- THEN startup succeeds; text watermark requests fail at runtime with
  `watermark_font_missing`
