//! Convenience logging macros built on `tracing`.

#[macro_export]
macro_rules! trace {
    ($($arg:tt)*) => {
        tracing::trace!($($arg)*)
    };
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        tracing::debug!($($arg)*)
    };
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        tracing::info!(
            module = module_path!(),
            file = file!(),
            line = line!(),
            $($arg)*
        )
    };
}

#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        tracing::warn!(
            module = module_path!(),
            file = file!(),
            line = line!(),
            $($arg)*
        )
    };
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        tracing::error!(
            module = module_path!(),
            file = file!(),
            line = line!(),
            $($arg)*
        )
    };
}
