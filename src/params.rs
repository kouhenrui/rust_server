//! `/img` 端点的查询字符串参数解析模块。
//!
//! 把 axum 通过 `Query` 提取器反序列化出来的原始字符串参数，
//! 校验后转换为强类型参数 [`ImgParams`]，供处理器和各处理环节使用。
//!
//! 解析策略是「宽容的」：未知字段会被忽略；一旦遇到语义错误
//! （缺 `src`、`w=0`、未知滤镜名等），立即返回 [`AppError::BadRequest`]。
//!
//! # 示例
//!
//! `GET /img?src=cat.jpg&w=200&h=200&fit=cover&filters=grayscale:blur(2)`
//!
//! [`AppError::BadRequest`]: crate::error::AppError::BadRequest

use crate::error::{AppError, AppResult};
use crate::proc::filter::FilterChain;
use crate::proc::transform::FitMode;
use serde::Deserialize;

/// 由 axum `Query` 提取器直接反序列化得到的「原始」查询参数。
///
/// 所有字段都是 `Option<...>`：缺失即代表调用方没传对应键。
/// 不在此结构上做语义校验 —— 调用 [`ImgParams::parse`] 完成转换。
#[derive(Debug, Default, Deserialize)]
pub struct ImgParamsRaw {
    /// `src`：源图标识。`http(s)://` URL、`file://` URI，或相对路径
    /// （拼上 `Config::local_source_root`）。
    pub src: Option<String>,

    /// `w`：目标宽度（像素，正整数）。
    pub w: Option<u32>,

    /// `h`：目标高度（像素，正整数）。
    pub h: Option<u32>,

    /// `fit`：`cover`（默认）| `contain` | `stretch`。参见 [`FitMode`]。
    pub fit: Option<String>,

    /// `crop`：显式裁切矩形，源像素坐标，格式 `x,y,w,h`。
    pub crop: Option<String>,

    /// `filters`：冒号分隔的滤镜链，例如 `grayscale:brightness(20):blur(2)`。
    pub filters: Option<String>,

    /// `watermark`：`Hello@10,10`（文本）或 `image:logo.png@10,10`（图像叠加）。
    pub watermark: Option<String>,

    /// `format`：输出格式覆盖。`png`（默认）| `jpeg` | `webp`。
    pub format: Option<String>,
}

/// 校验过的、强类型的参数集，供处理器和处理器链使用。
///
/// 与 [`ImgParamsRaw`] 的区别：本结构里的字段都已被解析成 enum、强类型
/// 数值，handler 不再需要自己处理字符串。
#[derive(Debug, Clone)]
pub struct ImgParams {
    /// 必填：源图标识。同 [`ImgParamsRaw::src`]。
    pub src: String,
    /// 目标尺寸（宽、高）。`None` 表示保持原图尺寸。
    pub target: Option<(u32, u32)>,
    /// 缩放策略。详见 [`FitMode`]。
    pub fit: FitMode,
    /// 可选：在缩放前先裁切源图。
    pub crop: Option<CropRect>,
    /// 滤镜链。按声明顺序依次应用。
    pub filters: FilterChain,
    /// 可选：文本或图像水印。
    pub watermark: Option<WatermarkSpec>,
    /// 输出编码格式。详见 [`OutputFormat`]。
    pub format: OutputFormat,
}

/// 源像素坐标系下的裁切矩形。
///
/// `(x, y)` 是左上角，`(w, h)` 是宽高，均为非负整数；
/// `w`、`h` 必须 > 0（在 [`parse_crop_rect`] 中校验）。
#[derive(Debug, Clone, Copy)]
pub struct CropRect {
    /// 左上角 X 坐标。
    pub x: u32,
    /// 左上角 Y 坐标。
    pub y: u32,
    /// 裁切宽度。
    pub w: u32,
    /// 裁切高度。
    pub h: u32,
}

/// 水印规格。
///
/// - `Text`：使用 `THUMBOR_WATERMARK_FONT` 指向的 TTF 字体渲染文本。
/// - `Image`：把 `path` 指向的图像作为不透明 / 半透明层叠加。
///
/// 两种情况下，`(x, y)` 都是水印**左上角**在结果图中的像素坐标，
/// 可为负（部分在画布外）也可能为 0。
#[derive(Debug, Clone)]
pub enum WatermarkSpec {
    /// 文本水印。
    Text {
        /// 要绘制的字符串。UTF-8 任意字符。
        text: String,
        /// 左上角 X 坐标。
        x: i32,
        /// 左上角 Y 坐标。
        y: i32,
    },
    /// 图像水印。
    Image {
        /// 图像路径，相对于 [`Config::local_source_root`] 解析。
        path: String,
        /// 左上角 X 坐标。
        x: i32,
        /// 左上角 Y 坐标。
        y: i32,
    },
}

/// 输出编码格式。
///
/// 由 `?format=` 查询参数决定；未传或为空时默认 [`OutputFormat::Png`]。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    /// PNG（有损/无损，alpha 透明通道）。
    Png,
    /// JPEG（有损，不支持 alpha）。
    Jpeg,
    /// WebP（当前实现为无损）。
    Webp,
}

impl OutputFormat {
    /// 返回该格式对应的 HTTP `Content-Type` 头取值。
    ///
    /// 返回的是 `&'static str`，可直接传给 `HeaderValue::from_static`。
    pub fn content_type(self) -> &'static str {
        match self {
            OutputFormat::Png => "image/png",
            OutputFormat::Jpeg => "image/jpeg",
            OutputFormat::Webp => "image/webp",
        }
    }
}

impl ImgParams {
    /// 把「原始」参数 [`ImgParamsRaw`] 解析成校验过的强类型参数。
    ///
    /// **为什么 `parse` 一次吞掉所有错误而不是让 handler 分头校验：**
    /// 一次解析一次性把 7 个字段映射到 enum / 强类型数值，
    /// 后续 transform/filter/watermark 阶段都不再需要处理字符串。
    /// 「集中校验 vs 分散校验」是这种 API 的核心抉择 —— 选集中是因为
    /// 字段之间有隐性关联（比如 `w=0` 与「用户是否真的想要缩放」是耦合的），
    /// handler 端不好做。
    ///
    /// # 参数
    ///
    /// * `raw` —— 由 axum `Query` 提取器反序列化得到的原始参数。
    ///
    /// # 错误
    ///
    /// 任意字段校验失败都会返回 [`AppError::BadRequest`]；
    /// 错误信息中包含原始字符串，便于调用方定位。
    ///
    /// # 示例
    ///
    /// ```
    /// use thumbor::params::{ImgParams, ImgParamsRaw};
    /// use thumbor::proc::transform::FitMode;
    ///
    /// let raw = ImgParamsRaw {
    ///     src: Some("cat.jpg".into()),
    ///     w: Some(200),
    ///     fit: Some("cover".into()),
    ///     ..Default::default()
    /// };
    /// let params = ImgParams::parse(raw).unwrap();
    /// assert_eq!(params.src, "cat.jpg");
    /// assert_eq!(params.target, Some((200, 0)));
    /// assert_eq!(params.fit, FitMode::Cover);
    /// ```
    pub fn parse(raw: ImgParamsRaw) -> AppResult<Self> {
        // 1. src 必填且非空
        let src = raw
            .src
            .filter(|s| !s.is_empty())
            .ok_or_else(|| AppError::BadRequest("missing required 'src'".into()))?;

        // 2. 解析 w/h：任一为 0 直接拒绝；至少要给一个
        let target = match (raw.w, raw.h) {
            (Some(0), _) | (_, Some(0)) => {
                return Err(AppError::BadRequest("w and h must be > 0".into()));
            }
            (w, h) if w.is_some() || h.is_some() => Some((w.unwrap_or(0), h.unwrap_or(0))),
            _ => None,
        };
        // 上面用 (0, 0) 作为「只给一个维度」的占位，这里再拦一次
        if let Some((0, 0)) = target {
            return Err(AppError::BadRequest("w and h must be > 0".into()));
        }

        // 3. fit 字符串 → enum
        let fit = match raw.fit.as_deref() {
            None | Some("cover") | Some("") => FitMode::Cover,
            Some("contain") => FitMode::Contain,
            Some("stretch") => FitMode::Stretch,
            Some(other) => {
                return Err(AppError::BadRequest(format!(
                    "unknown fit mode '{other}'"
                )))
            }
        };

        // 4-6. crop / filters / watermark 三个可选项
        let crop = raw
            .crop
            .as_deref()
            .map(parse_crop_rect)
            .transpose()?;

        let filters = raw
            .filters
            .as_deref()
            .map(FilterChain::parse)
            .transpose()?
            .unwrap_or_default();

        let watermark = raw
            .watermark
            .as_deref()
            .map(parse_watermark)
            .transpose()?;

        // 7. format 字符串 → enum
        let format = match raw.format.as_deref() {
            None | Some("png") | Some("") => OutputFormat::Png,
            Some("jpeg") | Some("jpg") => OutputFormat::Jpeg,
            Some("webp") => OutputFormat::Webp,
            Some(other) => {
                return Err(AppError::BadRequest(format!(
                    "unknown format '{other}'"
                )))
            }
        };

        Ok(Self {
            src,
            target,
            fit,
            crop,
            filters,
            watermark,
            format,
        })
    }
}

/// 把 `crop` 字符串解析成 [`CropRect`]。
///
/// **为什么不允许空格 + 维度顺序固定 `x,y,w,h`：** query 字符串里
/// 空格会被 `+` 或 `%20` 表示，下游解析时已经 normalize 过；统一
/// `x,y,w,h` 顺序是为了和 thumbor / imagemagick 这类老工具的语法兼容，
/// 客户端复用同一套 helper 就能复用。
///
/// # 参数
///
/// * `s` —— 形如 `"10,20,400,400"` 的 4 段逗号分隔字符串，前后空白会被 trim。
///
/// # 错误
///
/// - 段数 ≠ 4 → [`AppError::BadRequest`]
/// - 任意段不是非负整数 → [`AppError::BadRequest`]
/// - `w` 或 `h` 为 0 → [`AppError::BadRequest`]
fn parse_crop_rect(s: &str) -> AppResult<CropRect> {
    let parts: Vec<&str> = s.split(',').map(str::trim).collect();
    if parts.len() != 4 {
        return Err(AppError::BadRequest(format!(
            "crop must be 'x,y,w,h', got '{s}'"
        )));
    }
    let nums: Result<Vec<u32>, _> = parts.iter().map(|p| p.parse::<u32>()).collect();
    let nums = nums.map_err(|_| AppError::BadRequest(format!("crop has non-integer: '{s}'")))?;
    if nums[2] == 0 || nums[3] == 0 {
        return Err(AppError::BadRequest("crop w/h must be > 0".into()));
    }
    Ok(CropRect {
        x: nums[0],
        y: nums[1],
        w: nums[2],
        h: nums[3],
    })
}

/// 把 `watermark` 字符串解析成 [`WatermarkSpec`]。
///
/// **为什么用 `rsplit_once('@')` 而不是 `split_once('@')`：**
/// 文本本身可能含 `@`（比如邮箱风格的水印 `support@acme.com@10,10`），
/// 从右往左切坐标能保证前面 `@` 都不被切错。`image:` 前缀单独
/// 识别是另一种语法扩展 —— 文本和图像水印用同样的 `@x,y` 语法，
/// 用前缀区分种类，调用方不用切换参数。
///
/// # 参数
///
/// * `s` —— 查询参数 `watermark` 的原始字符串。
///
/// # 错误
///
/// - 缺少 `@` 分隔符 → [`AppError::BadRequest`]
/// - `@` 后坐标不是 `x,y` 两段 → [`AppError::BadRequest`]
/// - `x` 或 `y` 非整数 → [`AppError::BadRequest`]
/// - 文本 / 路径为空 → [`AppError::BadRequest`]
fn parse_watermark(s: &str) -> AppResult<WatermarkSpec> {
    // 优先识别 `image:` 前缀，其余一律当作文本水印处理
    let (kind, rest) = if let Some(stripped) = s.strip_prefix("image:") {
        ("image", stripped)
    } else {
        ("text", s)
    };
    // 从右往左切一刀，把坐标部分单独拿出来
    let (head, coords) = rest
        .rsplit_once('@')
        .ok_or_else(|| AppError::BadRequest("watermark must end with '@x,y'".into()))?;
    let pos: Vec<&str> = coords.split(',').map(str::trim).collect();
    if pos.len() != 2 {
        return Err(AppError::BadRequest("watermark coords must be 'x,y'".into()));
    }
    let x: i32 = pos[0].parse().map_err(|_| AppError::BadRequest("bad watermark x".into()))?;
    let y: i32 = pos[1].parse().map_err(|_| AppError::BadRequest("bad watermark y".into()))?;
    if head.is_empty() {
        return Err(AppError::BadRequest("watermark text/path is empty".into()));
    }
    Ok(if kind == "image" {
        WatermarkSpec::Image { path: head.to_string(), x, y }
    } else {
        WatermarkSpec::Text { text: head.to_string(), x, y }
    })
}
