# Image Request Parameters

## Purpose

Define how `/img` query strings and protobuf `ImageRequest` fields are parsed
into the shared [`ImgParams`] type used by the processing pipeline.

## Requirements

### Requirement: Shared validation via build

`params::ImgParams::build` MUST centralize validation for both GET and POST
paths: non-empty `src`, reject `w=0` or `h=0`, filter zero-sized crop rects.

`ImgParams::parse` MUST convert query strings then call `build`.
`controller/img.rs::img_request_to_params` MUST map proto fields then call
`build`.

#### Scenario: query and build agree

- GIVEN equivalent query params and explicit `build` arguments
- WHEN both produce `ImgParams`
- THEN `src`, `target`, `fit`, and `format` match

### Requirement: Query parameter parsing

`params::ImgParams::parse` MUST convert `ImgParamsRaw` (axum `Query`) into
validated `ImgParams`. Unknown query keys MUST be ignored. Semantic errors MUST
return `AppError::BadRequest`.

Supported query keys: `src`, `w`, `h`, `fit`, `crop`, `filters`, `watermark`,
`format`.

#### Scenario: missing src

- GIVEN a `GET /img` request without `src`
- WHEN `ImgParams::parse` runs
- THEN the error is `AppError::BadRequest`

#### Scenario: zero dimension

- GIVEN `w=0` or `h=0`
- WHEN `ImgParams::parse` runs
- THEN the error is `AppError::BadRequest`

#### Scenario: fit modes

- GIVEN `fit` is absent, `cover`, `contain`, or `stretch`
- WHEN parsing succeeds
- THEN `ImgParams.fit` is the corresponding `FitMode`

#### Scenario: crop rectangle

- GIVEN `crop=x,y,w,h` with positive `w` and `h`
- WHEN parsing succeeds
- THEN `ImgParams.crop` is `Some(CropRect { x, y, w, h })`

#### Scenario: watermark text

- GIVEN `watermark=Hello@10,20`
- WHEN parsing succeeds
- THEN `ImgParams.watermark` is `Text { text: "Hello", x: 10, y: 20 }`

#### Scenario: watermark image

- GIVEN `watermark=image:logo.png@5,5`
- WHEN parsing succeeds
- THEN `ImgParams.watermark` is `Image { path: "logo.png", x: 5, y: 5 }`

### Requirement: Protobuf conversion

`controller/img.rs` MUST map `api::ImageRequest` to `ImgParams` via
`img_request_to_params`, delegating validation to `ImgParams::build`.

#### Scenario: shared pipeline

- GIVEN equivalent GET query params and POST `ImageRequest` fields
- WHEN both reach `process_image`
- THEN the same transform/filter/watermark/encode steps run

### Requirement: Cache key

`ImgParams::cache_key` MUST produce a stable string from normalized transform
parameters for `/img` result caching (see `cache-backend` spec).

#### Scenario: identical params same key

- GIVEN two `ImgParams` with the same `src`, dimensions, fit, crop, filters, watermark, and format
- WHEN `cache_key` is called on each
- THEN both keys are equal

### Requirement: Filter chain delegation

The `filters` query value MUST be parsed by `FilterChain::parse` in
`proc/filter.rs` (see `image-filters` spec).

#### Scenario: filters parsed into chain

- GIVEN query `filters=grayscale:blur(2.0)`
- WHEN `ImgParams::parse` succeeds
- THEN `params.filters` contains two filters in declaration order
