use clap::ValueEnum;

#[derive(Debug, Clone, Copy, serde::Deserialize, ValueEnum, Default)]
pub enum LogLevel {
    /// A level lower than all log levels, intended to disable logging.
    #[serde(alias = "off")]
    Off,
    /// Print only errors.
    #[serde(alias = "error")]
    Error,
    /// Print warnings and errors.
    #[serde(alias = "warn")]
    Warn,
    /// Print info, warnings, and errors.
    #[serde(alias = "info")]
    #[default]
    Info,
    /// Print debug, info, warnings, and errors.
    #[serde(alias = "debug")]
    Debug,
    /// Print all log messages.
    #[serde(alias = "trace")]
    Trace,
}

impl From<LogLevel> for log::LevelFilter {
    fn from(level: LogLevel) -> log::LevelFilter {
        match level {
            LogLevel::Off => log::LevelFilter::Off,
            LogLevel::Error => log::LevelFilter::Error,
            LogLevel::Warn => log::LevelFilter::Warn,
            LogLevel::Info => log::LevelFilter::Info,
            LogLevel::Debug => log::LevelFilter::Debug,
            LogLevel::Trace => log::LevelFilter::Trace,
        }
    }
}

#[cfg(not(debug_assertions))]
pub async fn set_log_output(level: Option<LogLevel>, output: &Vec<String>) -> anyhow::Result<()> {
    let level = level.unwrap_or_default();
    let output = match output.is_empty() {
        true => &vec!["stdout".to_string()],
        false => output,
    };
    let mut builder = fern::Dispatch::new()
        // Perform allocation-free log formatting
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(level.into());

    for file in output {
        match file.as_str() {
            "stdout" => builder = builder.chain(std::io::stdout()),
            file => {
                // create the directory if it doesn't exist
                if let Some(parent) = std::path::Path::new(file).parent() {
                    tokio::fs::create_dir_all(parent).await?;
                }
                builder = builder.chain(fern::log_file(file)?)
            }
        }
    }
    // Apply globally
    builder.apply()?;

    Ok(())
}
