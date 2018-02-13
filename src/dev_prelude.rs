pub use expect_macro::*;

#[allow(unused_imports)]
pub use ergo::*;
#[allow(unused_imports)]
pub use quicli::prelude::*;
pub use ordermap::*;

#[macro_export]
macro_rules! work_dir { [$cmd:expr] => {{
    match $cmd.work_dir {
        Some(ref d) => PathDir::new(d),
        None => PathDir::current_dir(),
    }?
}}}

#[macro_export]
macro_rules! set_log_verbosity { [$cmd:expr] => {{
    set_log_verbosity("art", $cmd.verbosity)?;
}}}

/// Set the logs verbosity based on an integer value:
///
/// - `0`: error
/// - `1`: warn
/// - `2`: info
/// - `3`: debug
/// - `>=4`: trace
///
/// This is used in the [`main!`] macro. You should typically use that instead.
///
/// [`main!`]: macro.main.html
pub fn set_log_verbosity(pkg: &str, verbosity: u64) -> Result<()> {
    let log_level = match verbosity {
        0 => LogLevel::Error,
        1 => LogLevel::Warn,
        2 => LogLevel::Info,
        3 => LogLevel::Debug,
        _ => LogLevel::Trace,
    }.to_level_filter();

    LoggerBuiler::new()
        .filter(Some(pkg), log_level)
        .filter(None, LogLevel::Warn.to_level_filter())
        .try_init()?;
    Ok(())
}
