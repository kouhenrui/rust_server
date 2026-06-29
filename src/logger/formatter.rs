//! Log line formatting helpers.

use time::macros::format_description;
use tracing::Subscriber;
use tracing_subscriber::fmt::{self, format::FmtSpan};
use tracing_subscriber::fmt::time::LocalTime;
use tracing_subscriber::layer::Layer;
use tracing_subscriber::registry::LookupSpan;

use super::config::LoggerConfig;

/// Build the default `fmt` layer for the global subscriber.
pub fn fmt_layer<S>(cfg: &LoggerConfig) -> impl Layer<S> + Send + Sync + 'static
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    let timer = LocalTime::new(format_description!(
        "[year]-[month]-[day] [hour]:[minute]:[second]"
    ));
    fmt::layer()
        .with_timer(timer)
        .with_span_events(FmtSpan::NONE)
        .with_target(cfg.show_target)
}
