use std::error::Error;
use std::ops::DerefMut;
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;
use std::time::SystemTime;
use log::{Level, Log, Metadata, Record};
use crate::logging::ring_buffer::RingBuffer;

mod ring_buffer;



#[derive(Debug, Clone)]
pub struct LogEntry {
    timestamp: SystemTime,
    log_level: Level,
    target: String,
    message: String,
    module_path: Option<String>,
    file: Option<String>,
    line: Option<u32>,
}

impl LogEntry {
    pub fn timestamp(&self) -> SystemTime {
        self.timestamp.clone()
    }
    pub fn log_level(&self) -> Level {
        self.log_level
    }
    pub fn target(&self) -> &str {
        &self.target
    }
    pub fn message(&self) -> &str {
        &self.message
    }
    pub fn module_path(&self) -> Option<&str> {
        self.module_path.as_deref()
    }
    pub fn file(&self) -> Option<&str> {
        self.file.as_deref()
    }
    pub fn line(&self) -> Option<u32> {
        self.line
    }

    pub fn format(&self, time_fmt_string: &str) -> Result<(String, String, String, String), Box<dyn Error>> {

        let timestamp = if time_fmt_string.is_empty() {
            format!("{}", self.timestamp.duration_since(SystemTime::UNIX_EPOCH)?.as_millis())
        } else {
            let time = time_format::from_system_time(self.timestamp.clone())?;
            time_format::strftime_local(time_fmt_string, time)?
        };

        let module = match &self.module_path {
            Some(module_path) => format!("({})", module_path),
            None => String::new(),
        };
        let file = match (&self.file, &self.line) {
            (Some(file), Some(line)) => format!("({}:{})", file, line),
            (Some(file), None) => format!("({})", file),
            _ => String::new(),
        };

        let message = self.message.clone();

        Ok((timestamp, module, file, message))
    }
}

impl From<&Record<'_>> for LogEntry {
    fn from(record: &Record) -> Self {
        let timestamp = SystemTime::now();
        let target = record.target().to_string();
        let message = record.args().to_string();
        let module_path = record.module_path().map(|s| s.to_string());
        let file = record.file().map(|s| s.to_string());
        let line = record.line();
        LogEntry {
            timestamp,
            log_level: record.level(),
            target,
            message,
            module_path,
            file,
            line,
        }
    }
}

#[derive(Clone)]
pub struct LogKind {
    buffer: Arc<Mutex<RingBuffer>>,
}

impl LogKind {
    pub fn new(capacity: usize) -> Self {
        LogKind {
            buffer: Arc::new(Mutex::new(RingBuffer::new(capacity)))
        }
    }

    pub fn buffer(&self) -> impl DerefMut<Target = RingBuffer> + '_ {
        self.buffer.blocking_lock()
    }

    pub async fn buffer_async(&self) -> impl DerefMut<Target = RingBuffer> + '_ {
        self.buffer.lock().await
    }
}

static LOGGER: OnceLock<Logger> = OnceLock::new();

#[derive(Clone)]
pub struct Logger {
    trace: LogKind,
    debug: LogKind,
    info: LogKind,
    warn: LogKind,
    error: LogKind,
}

impl Logger {
    fn new(capacity: usize) -> Logger {
        let trace = LogKind::new(capacity);
        let debug = LogKind::new(capacity);
        let info = LogKind::new(capacity);
        let warn = LogKind::new(capacity);
        let error = LogKind::new(capacity);

        Logger {
            trace,
            debug,
            info,
            warn,
            error,
        }
    }

    /// Creates a new Rust logger and installs it.
    ///
    /// `capacity` is the max size of stored logs for each kind of log.
    ///
    /// `panics` when the logger has already been installed.
    pub fn install_logger(capacity: usize) {
        let logger = Logger::new(capacity);
        match LOGGER.set(logger.clone()) {
            Ok(_) => {}
            Err(_) => panic!("Logger already initialized"),
        }
        log::set_boxed_logger(Box::new(logger)).unwrap();
    }

    fn trace(&self) -> LogKind {
        self.trace.clone()
    }
    fn debug(&self) -> LogKind {
        self.debug.clone()
    }
    fn info(&self) -> LogKind {
        self.info.clone()
    }
    fn warn(&self) -> LogKind {
        self.warn.clone()
    }
    fn error(&self) -> LogKind {
        self.error.clone()
    }

    fn get_logger() -> Logger {
        LOGGER.get().expect("logger was not initialized").clone()
    }

    pub fn log_trace(record: &Record) {
        let logger = Logger::get_logger();
        logger.trace().buffer().push(LogEntry::from(record));
    }
    pub async fn log_trace_async(record: &Record<'_>) {
        let logger = Logger::get_logger();
        logger.error().buffer_async().await.push(LogEntry::from(record));
    }
    pub fn log_debug(record: &Record) {
        let logger = Logger::get_logger();
        logger.debug().buffer().push(LogEntry::from(record));
    }
    pub async fn log_debug_async(record: &Record<'_>) {
        let logger = Logger::get_logger();
        logger.debug().buffer_async().await.push(LogEntry::from(record));
    }
    pub fn log_info(record: &Record) {
        let logger = Logger::get_logger();
        logger.info().buffer().push(LogEntry::from(record));
    }
    pub async fn log_info_async(record: &Record<'_>) {
        let logger = Logger::get_logger();
        logger.info().buffer_async().await.push(LogEntry::from(record));
    }
    pub fn log_warn(record: &Record) {
        let logger = Logger::get_logger();
        logger.warn().buffer().push(LogEntry::from(record));
    }
    pub async fn log_warn_async(record: &Record<'_>) {
        let logger = Logger::get_logger();
        logger.warn().buffer_async().await.push(LogEntry::from(record));
    }
    pub fn log_error(record: &Record) {
        let logger = Logger::get_logger();
        logger.error().buffer().push(LogEntry::from(record));
    }
    pub async fn log_error_async(record: &Record<'_>) {
        let logger = Logger::get_logger();
        logger.error().buffer_async().await.push(LogEntry::from(record));
    }

    pub fn trace_logs() -> Vec<LogEntry> {
        let logger = Logger::get_logger();
        logger.trace().buffer.blocking_lock().to_vec()
    }
    pub async fn trace_logs_async() -> Vec<LogEntry> {
        let logger = Logger::get_logger();
        logger.trace().buffer.lock().await.to_vec()
    }
    pub fn debug_logs() -> Vec<LogEntry> {
        let logger = Logger::get_logger();
        logger.debug().buffer.blocking_lock().to_vec()
    }
    pub async fn debug_logs_async() -> Vec<LogEntry> {
        let logger = Logger::get_logger();
        logger.debug().buffer.lock().await.to_vec()
    }
    pub fn info_logs() -> Vec<LogEntry> {
        let logger = Logger::get_logger();
        logger.info().buffer.blocking_lock().to_vec()
    }
    pub async fn info_logs_async() -> Vec<LogEntry> {
        let logger = Logger::get_logger();
        logger.info().buffer.lock().await.to_vec()
    }
    pub fn warn_logs() -> Vec<LogEntry> {
        let logger = Logger::get_logger();
        logger.warn().buffer.blocking_lock().to_vec()
    }
    pub async fn warn_logs_async() -> Vec<LogEntry> {
        let logger = Logger::get_logger();
        logger.warn().buffer.lock().await.to_vec()
    }
    pub fn error_logs() -> Vec<LogEntry> {
        let logger = Logger::get_logger();
        logger.error().buffer.blocking_lock().to_vec()
    }
    pub async fn error_logs_async() -> Vec<LogEntry> {
        let logger = Logger::get_logger();
        logger.error().buffer.lock().await.to_vec()
    }

    /// Fetches all logs from the logger.
    ///
    /// There is no ordering between log types
    pub fn all_logs() -> Vec<LogEntry> {
        let mut output = Vec::new();
        let logger = Logger::get_logger();
        output.append(&mut logger.trace().buffer().to_vec());
        output.append(&mut logger.debug().buffer().to_vec());
        output.append(&mut logger.info().buffer().to_vec());
        output.append(&mut logger.warn().buffer().to_vec());
        output.append(&mut logger.error().buffer().to_vec());

        output.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        output
    }
}

impl Log for Logger {
    fn enabled(&self, _: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let log_kind = match record.level() {
            Level::Trace => self.trace(),
            Level::Debug => self.debug(),
            Level::Info => self.info(),
            Level::Warn => self.warn(),
            Level::Error => self.error(),
        };
        loop {
            log_kind.buffer().push(LogEntry::from(record));
        }
    }

    fn flush(&self) {
        // does nothing
    }
}