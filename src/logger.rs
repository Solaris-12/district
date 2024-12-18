use colored::*;

#[derive(Debug)]
pub enum LogLevel {
    Dev = 9,
    Debug = 0,
    Info = 1,
    Warning = 2,
    Error = 3,
}

impl ToString for LogLevel {
    fn to_string(&self) -> String {
        match self {
            LogLevel::Dev => String::from("DEV"),
            LogLevel::Debug => String::from("DEBUG"),
            LogLevel::Info => String::from("INFO"),
            LogLevel::Warning => String::from("WARN"),
            LogLevel::Error => String::from("ERROR"),
        }
    }
}

pub struct Logger {
    level: LogLevel,
}

impl Logger {
    pub fn new(level: LogLevel) -> Self {
        Logger { level }
    }

    pub fn log<T: std::fmt::Debug + ToString>(&self, message: T, caller: &str) {
        let level_color = match self.level {
            LogLevel::Dev => "magenta",
            LogLevel::Debug => "green",
            LogLevel::Info => "blue",
            LogLevel::Warning => "yellow",
            LogLevel::Error => "red",
        };

        let display_message: String = message.to_string();
        println!(
            "[{:<30}] {:<6}: {}",
            caller.truecolor(120, 120, 120),
            self.level.to_string().color(level_color).bold(),
            display_message
        );
    }
}

#[macro_export]
macro_rules! log_x {
    ($level:expr, $message:expr) => {{
        let caller = file!();
        let logger = Logger::new($level);
        logger.log($message, caller);
    }};
}

#[macro_export]
macro_rules! log_d {
    ($message:expr) => {
        log_x!(LogLevel::Debug, $message)
    };
}

#[macro_export]
macro_rules! log_i {
    ($message:expr) => {
        log_x!(LogLevel::Info, $message)
    };
}

#[macro_export]
macro_rules! log_w {
    ($message:expr) => {
        log_x!(LogLevel::Warning, $message)
    };
}

#[macro_export]
macro_rules! log_e {
    ($message:expr) => {
        log_x!(LogLevel::Error, $message)
    };
}
