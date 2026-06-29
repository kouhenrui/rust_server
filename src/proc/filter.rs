//! Filter chain. Syntax: colon-separated `name(args)` invocations.
//!
//!   `grayscale`
//!   `brightness(20)`     // -100..100
//!   `contrast(15)`       // -100..100
//!   `blur(2.0)`          // gaussian sigma, pixels
//!
//! Adding a new filter means: implement [`Filter::apply`], add a variant to
//! [`Filter`], and extend [`FilterChain::parse_one`]. See `AGENTS.md`.

use crate::error::{AppError, AppResult};
use image::DynamicImage;

/// One parsed filter invocation.
#[derive(Debug, Clone, PartialEq)]
pub enum Filter {
    Grayscale,
    Brightness(i32),
    Contrast(i32),
    Blur(f32),
}

impl Filter {
    /// 把单个滤镜应用到位图上。
    ///
    /// `Brightness` / `Contrast` 显式转 `ImageRgba8` 是因为 `image` crate
    /// 提供的 `brighten_in_place` / `contrast_in_place` 是 `RgbaImage` 的
    /// 关联方法 —— `DynamicImage` 的 in-place 版本并不存在。中间做一次
    /// 转换的代价是颜色格式的归一化（grayscale/RGB → RGBA），换来
    /// 调用接口干净。
    pub fn apply(self, img: &mut DynamicImage) {
        match self {
            Filter::Grayscale => {
                *img = img.grayscale();
            }
            Filter::Brightness(amt) => {
                // image::imageops::brighten works in-place on RGBA buffers.
                let mut rgba = img.to_rgba8();
                image::imageops::colorops::brighten_in_place(&mut rgba, amt);
                *img = DynamicImage::ImageRgba8(rgba);
            }
            Filter::Contrast(amt) => {
                let mut rgba = img.to_rgba8();
                image::imageops::colorops::contrast_in_place(&mut rgba, amt as f32);
                *img = DynamicImage::ImageRgba8(rgba);
            }
            Filter::Blur(sigma) => {
                *img = img.blur(sigma);
            }
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FilterChain {
    pub filters: Vec<Filter>,
}

impl FilterChain {
    /// 把 `grayscale:brightness(20):blur(2.0)` 这样的字符串解析成链。
    ///
    /// `split(':').map(str::trim).filter(!is_empty)` 是为了容忍
    /// `?filters=grayscale::blur(2)` 这种「多了冒号」的情况
    /// （前端拼 query 时偶有发生），而不会报奇怪的「unknown filter
    /// ''」错。空字符串直接返回空链 —— 调用方常常把 `?filters=` 当
    /// 「没传」用。
    pub fn parse(spec: &str) -> AppResult<Self> {
        let mut filters = Vec::new();
        for raw in spec.split(':').map(str::trim).filter(|s| !s.is_empty()) {
            filters.push(parse_one(raw)?);
        }
        Ok(Self { filters })
    }

    /// Apply every filter in declaration order.
    pub fn apply(self, img: &mut DynamicImage) {
        for f in self.filters {
            f.apply(img);
        }
    }
}

/// 解析单个 `name(arg,arg)` 片段。
///
/// 关键设计：**第一个 `(` 切一刀**，让 filter 名字里可以含括号以外的
/// 任意字符（虽然现在没用到）。但 args 里的逗号不能再包括号 —— 这是
/// 「简洁性 vs 完备性」的小取舍，目前滤镜参数都是简单数字，不需要嵌套
/// 表达式，留给未来扩展。
fn parse_one(token: &str) -> AppResult<Filter> {
    // Split on the first '(' so filter names with parens in args are preserved.
    let (name, args) = match token.find('(') {
        Some(i) => (&token[..i], Some(&token[i + 1..token.len() - 1])),
        None => (token, None),
    };
    let args = args.unwrap_or("");
    let parts: Vec<&str> = if args.is_empty() {
        Vec::new()
    } else {
        args.split(',').map(str::trim).collect()
    };
    Ok(match name {
        "grayscale" => Filter::Grayscale,
        "brightness" => {
            let v: i32 = exactly_one(&parts, name)?
                .parse()
                .map_err(|_| AppError::Filter(format!("brightness: bad int '{args}'")))?;
            Filter::Brightness(v)
        }
        "contrast" => {
            let v: i32 = exactly_one(&parts, name)?
                .parse()
                .map_err(|_| AppError::Filter(format!("contrast: bad int '{args}'")))?;
            Filter::Contrast(v)
        }
        "blur" => {
            let v: f32 = exactly_one(&parts, name)?
                .parse()
                .map_err(|_| AppError::Filter(format!("blur: bad float '{args}'")))?;
            // σ > 100 在像素尺度上已经看不出区别（整图糊成一片），
            // 拦掉是为了拒绝「手抖打错小数点」之类的明显 bug。
            if !(0.0..=100.0).contains(&v) {
                return Err(AppError::Filter(format!(
                    "blur sigma {v} out of range 0..=100"
                )));
            }
            Filter::Blur(v)
        }
        other => {
            return Err(AppError::Filter(format!("unknown filter '{other}'")));
        }
    })
}

/// 收口「arg 数量必须 = 1」这条规则。
///
/// 单独提一个 helper 是为了把错误信息统一到一处：所有「只要一个参数」
/// 的滤镜（目前全部）走这里，将来加 `watermark("text", size=10)` 这类
/// 多参滤镜时这个 helper 可以删掉，是临时性抽象。
fn exactly_one<'a>(parts: &'a [&'a str], name: &str) -> AppResult<&'a str> {
    match parts {
        [one] => Ok(one),
        _ => Err(AppError::Filter(format!(
            "filter '{name}' expects exactly one argument"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_chain() {
        let c = FilterChain::parse("grayscale:brightness(20):blur(1.5)").unwrap();
        assert_eq!(c.filters.len(), 3);
        assert_eq!(c.filters[0], Filter::Grayscale);
        assert_eq!(c.filters[1], Filter::Brightness(20));
        assert_eq!(c.filters[2], Filter::Blur(1.5));
    }

    #[test]
    fn rejects_unknown_filter() {
        let err = FilterChain::parse("sepia").unwrap_err();
        assert!(matches!(err, AppError::Filter(_)));
    }

    #[test]
    fn rejects_bad_arg_count() {
        assert!(FilterChain::parse("brightness(1,2)").is_err());
    }

    #[test]
    fn rejects_out_of_range_blur() {
        assert!(FilterChain::parse("blur(500)").is_err());
    }

    #[test]
    fn empty_string_yields_empty_chain() {
        let c = FilterChain::parse("").unwrap();
        assert!(c.filters.is_empty());
    }
}
