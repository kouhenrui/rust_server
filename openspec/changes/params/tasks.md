# Tasks: params

## 1. Parameter types

- [x] 1.1 Create `src/params.rs` with `ImgParamsRaw`, `ImgParams`, `CropRect`,
  `WatermarkSpec`, `OutputFormat`. (`src/params.rs`)
- [x] 1.2 `ImgParams::parse` validates `src`, dimensions, `fit`, `crop`,
  `filters`, `watermark`, `format`.
- [x] 1.3 `parse_crop_rect` enforces `x,y,w,h` with positive width/height.
- [x] 1.4 `parse_watermark` supports text (`text@x,y`) and image
  (`image:path@x,y`) via `rsplit_once('@')`.

## 2. Protobuf bridge

- [x] 2.1 `img_request_to_params` in `src/controller/img.rs` maps
  `api::ImageRequest` → `ImgParams`.
- [x] 2.2 Enum fields (`fit`, `format`) translated to internal types.

## 3. Wire into handlers

- [x] 3.1 `img_get` uses `Query<ImgParamsRaw>` + `ImgParams::parse`.
- [x] 3.2 `img_post` uses protobuf decode then `img_request_to_params`.

## 4. Verification

- [x] 4.1 Integration tests cover bad query (`w=0`, missing `src`) and protobuf
  validation errors. (`tests/integration.rs`)
