//! Resize, fit-mode, and explicit crop operations.

use crate::error::{AppError, AppResult};
use crate::params::CropRect;
use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView};

/// How a target `(w, h)` is applied when the aspect ratio of the source
/// differs from the target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FitMode {
    /// Crop the source to fill the target box, preserving aspect ratio.
    Cover,
    /// Fit the source inside the target box, preserving aspect ratio.
    Contain,
    /// Ignore aspect ratio, distort to exact dimensions.
    Stretch,
}

/// 把整个几何变换流水线串起来：先裁切、再缩放。
///
/// **顺序不是巧合** —— 裁切完再缩放，缩放要处理的像素数比「先缩放
/// 后裁切」少几倍。`cover` 模式本身在缩放内部还会再裁一次，但那是
/// 缩放到目标框时为了凑满的，发生在用户显式裁切之后，意义不同。
/// 失败立即返回（`crop_to` 的越界、`BadRequest` 的零维），整个请求
/// 走错误路径。
pub fn apply(
    img: &mut DynamicImage,
    crop: Option<CropRect>,
    target: Option<(u32, u32)>,
    fit: FitMode,
) -> AppResult<()> {
    if let Some(c) = crop {
        *img = crop_to(img, c)?;
    }
    if let Some((tw, th)) = target {
        if tw == 0 || th == 0 {
            return Err(AppError::BadRequest("w and h must be > 0".into()));
        }
        *img = resize_with_fit(img, tw, th, fit);
    }
    Ok(())
}

/// 显式裁切，返回新的 `DynamicImage`。
///
/// `crop_imm` 之后返回 `DynamicImage`（而不是改原图）是为了配合
/// `apply` 的 `*img = ...` 模式。`min(w - x)` 是为了越界时**截短**
/// 而不是 panic —— 调用方已经把整体合法性检查过了，但坐标踩在右下角
/// 时仍可能有 1-2 像素溢出，做个 saturating。
pub fn crop_to(img: &DynamicImage, rect: CropRect) -> AppResult<DynamicImage> {
    let (w, h) = img.dimensions();
    if rect.x >= w || rect.y >= h {
        return Err(AppError::BadRequest(format!(
            "crop origin ({},{}) outside {}x{} image",
            rect.x, rect.y, w, h
        )));
    }
    let crop_w = rect.w.min(w - rect.x);
    let crop_h = rect.h.min(h - rect.y);
    Ok(img.crop_imm(rect.x, rect.y, crop_w, crop_h))
}

/// 按 [`FitMode`] 把源图缩放到精确尺寸 `tw × th`。
///
/// `Lanczos3` 是 `image` crate 里质量最高的内置滤波器，
/// 比默认的 `Triangle` 锐利得多；CPU 成本约 2-3x，对于「图片处理
/// 服务」这个场景完全可以接受。`Contain` 走透明画布 + `overlay`
/// 是因为透明 `image` crate API 在「letterbox」上没有原语支持，
/// 自己造一个画布然后叠加比 hack `resize` 行为干净。
pub fn resize_with_fit(img: &DynamicImage, tw: u32, th: u32, fit: FitMode) -> DynamicImage {
    let (_sw, _sh) = img.dimensions();
    match fit {
        FitMode::Stretch => img.resize_exact(tw, th, FilterType::Lanczos3),
        FitMode::Cover => img.resize_to_fill(tw, th, FilterType::Lanczos3),
        FitMode::Contain => {
            // Letterbox onto a transparent canvas.
            let scaled = img.resize(tw, th, FilterType::Lanczos3);
            let (scaled_w, scaled_h) = scaled.dimensions();
            let mut canvas = DynamicImage::new_rgba8(tw, th);
            let off_x = (tw.saturating_sub(scaled_w)) / 2;
            let off_y = (th.saturating_sub(scaled_h)) / 2;
            image::imageops::overlay(&mut canvas, &scaled, off_x as i64, off_y as i64);
            canvas
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    fn sample_image(w: u32, h: u32) -> DynamicImage {
        let buf: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_fn(w, h, |x, y| Rgba([x as u8, y as u8, 128, 255]));
        DynamicImage::ImageRgba8(buf)
    }

    #[test]
    fn stretch_resizes_to_exact_target() {
        let mut img = sample_image(100, 50);
        apply(&mut img, None, Some((40, 40)), FitMode::Stretch).unwrap();
        assert_eq!(img.dimensions(), (40, 40));
    }

    #[test]
    fn cover_fills_target_box() {
        let mut img = sample_image(100, 50);
        apply(&mut img, None, Some((40, 40)), FitMode::Cover).unwrap();
        assert_eq!(img.dimensions(), (40, 40));
    }

    #[test]
    fn contain_letterboxes_to_target_canvas() {
        let mut img = sample_image(100, 50);
        apply(&mut img, None, Some((40, 40)), FitMode::Contain).unwrap();
        assert_eq!(img.dimensions(), (40, 40));
    }

    #[test]
    fn crop_runs_before_resize() {
        let mut img = sample_image(100, 100);
        let crop = CropRect {
            x: 10,
            y: 10,
            w: 20,
            h: 20,
        };
        apply(&mut img, Some(crop), Some((10, 10)), FitMode::Stretch).unwrap();
        assert_eq!(img.dimensions(), (10, 10));
    }

    #[test]
    fn apply_rejects_zero_target_dimension() {
        let mut img = sample_image(10, 10);
        let err = apply(&mut img, None, Some((0, 10)), FitMode::Cover).unwrap_err();
        assert!(matches!(err, AppError::BadRequest(_)));
    }
}
