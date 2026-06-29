//! Watermark overlays: text (rendered via `imageproc` + `ab_glyph`/rusttype)
//! or image. The font is loaded lazily from `Config::watermark_font` via
//! `FontCache`.

use crate::error::{AppError, AppResult};
use crate::params::WatermarkSpec;
use crate::state::AppState;
use ab_glyph::{Font, FontVec, PxScale, ScaleFont};
use image::imageops::overlay;
use image::{DynamicImage, GenericImageView, Rgba};
use imageproc::drawing::draw_text_mut;

/// 按 [`WatermarkSpec`] 把水印叠加到 `img` 上。
///
/// 是 handler 流水线的**最后一步**：所有几何变换、滤镜都做完之后
/// 才贴水印 —— 否则水印也会被滤镜处理掉（`blur` 一下字就没了）。
pub fn apply(img: &mut DynamicImage, spec: &WatermarkSpec, state: &AppState) -> AppResult<()> {
    match spec {
        WatermarkSpec::Text { text, x, y } => draw_text_watermark(img, text, *x, *y, state),
        WatermarkSpec::Image { path, x, y } => draw_image_watermark(img, path, *x, *y, state),
    }
}

/// 渲染并叠加文本水印。
///
/// 关键设计：**先画到与文字等大的临时透明画布上，再 `overlay` 到主图**。
/// 原因是 `imageproc::drawing::draw_text_mut` 不会自动 clip 出界部分，
/// 在大画布上直接画会出现越界写入 —— 自己造一个尺寸严丝合缝的小画布
/// 之后 `overlay` 就能让 `image::imageops::overlay` 帮我们处理越界。
///
/// 字体大小写死 32.0 px 是「有意识的简化」：URL 协议里要再加 `size=N`
/// 是 API 演化的下一步；当前用例下用户只用它来打个版权水印，
/// 32px 在大多数缩略图上够看了。
fn draw_text_watermark(
    img: &mut DynamicImage,
    text: &str,
    x: i32,
    y: i32,
    state: &AppState,
) -> AppResult<()> {
    let path = state
        .config
        .watermark_font
        .as_ref()
        .ok_or(AppError::WatermarkFontMissing)?;
    let bytes = state
        .fonts
        .get(path)
        .ok_or_else(|| AppError::Internal(format!("font not readable: {}", path.display())))?;
    let font = FontVec::try_from_vec(bytes.to_vec())
        .map_err(|_| AppError::Internal("font parse failed".into()))?;

    let scale = PxScale::from(32.0);
    let scaled = font.as_scaled(scale);
    let ascent = scaled.ascent();
    let descent = scaled.descent();
    let text_h = (ascent - descent).ceil() as i32;

    // Estimate width by summing glyph advances; cheap and avoids touching `ab_glyph::Glyph` here.
    let mut width = 0.0f32;
    for ch in text.chars() {
        width += scaled.h_advance(font.glyph_id(ch));
    }
    let text_w = width.ceil() as i32;

    // Render onto a transparent canvas the size of the text, then overlay.
    let mut text_img = DynamicImage::new_rgba8(text_w.max(1) as u32, text_h.max(1) as u32);
    draw_text_mut(
        &mut text_img,
        Rgba([255, 255, 255, 255]),
        0,
        0,
        scale,
        &font,
        text,
    );
    overlay(img, &text_img, x as i64, y as i64);
    Ok(())
}

/// 加载并叠加图像水印。
///
/// 路径解析复用 `local_source_root`：水印图跟普通源图走同一套「相对路径」
/// 语义，运维只配一个 root 就够了。**不能**走 `http(s)://`：水印文件
/// 是部署期打包进去的（CDN 拉上游会引入不稳定和版权风险），
/// 路径必须是本地的。
fn draw_image_watermark(
    img: &mut DynamicImage,
    path: &str,
    x: i32,
    y: i32,
    state: &AppState,
) -> AppResult<()> {
    let resolved = match &state.config.local_source_root {
        Some(root) => root.join(path),
        None => std::path::PathBuf::from(path),
    };
    let bytes = std::fs::read(&resolved)
        .map_err(|_| AppError::SourceNotFound(resolved.display().to_string()))?;
    let overlay_img = crate::source::decode(&bytes)?;
    let (w, h) = overlay_img.dimensions();
    if w == 0 || h == 0 {
        return Err(AppError::BadRequest("watermark image has zero size".into()));
    }
    overlay(img, &overlay_img, x as i64, y as i64);
    Ok(())
}
