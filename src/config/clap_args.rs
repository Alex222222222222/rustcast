use clap::Parser;

use super::log_level::LogLevel;

/// RustCast is a robust and efficient Shoutcast streaming server written in Rust.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct ClapArgs {
    /// The path to the configuration file
    pub config: String,

    /// Log level.
    /// The log level specified here will override the log level in the configuration file.
    #[arg(short, long, value_enum)]
    pub log_level: Option<LogLevel>,

    /// Log files. Can be specified multiple times.
    /// "stdout" are special values that will log to your terminal.
    /// If not specified, logs will only be written to stdout.
    /// If specified, the `log_file` field in the configuration file will be ignored.
    #[arg(long)]
    pub log_file: Vec<String>,
}
