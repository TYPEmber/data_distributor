use log::{debug, error, info, trace, warn};
use log::{Level, Metadata, Record};

struct AsyncLogger {
    tx: tokio::sync::broadcast::Sender<String>,
}

impl AsyncLogger {
    pub fn new() -> Self {
        let (tx, _) = tokio::sync::broadcast::channel::<String>(20);

        AsyncLogger { tx }
    }
}

impl log::Log for AsyncLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        //println!("{:?}", record);
        if self.enabled(record.metadata()) {
            self.tx
                .send(if record.level() == Level::Error {
                    format!(
                        "[{}]{}[{}][{}][{}]",
                        record.level(),
                        record.args(),
                        record.module_path().unwrap_or("<unamed>"),
                        record.file().unwrap_or("<unamed>"),
                        record.line().unwrap_or(0)
                    )
                } else {
                    format!("[{}] {}", record.level(), record.args())
                })
                .unwrap();
        }
    }

    fn flush(&self) {}
}

use log::{LevelFilter, SetLoggerError};

lazy_static::lazy_static! {
    static ref LOGGER: AsyncLogger = AsyncLogger::new();
    static ref CH:( tokio::sync::broadcast::Sender<()>,  tokio::sync::broadcast::Receiver<()>) =  tokio::sync::broadcast::channel(1);
}

pub fn init() -> Result<tokio::sync::broadcast::Sender<()>, SetLoggerError> {
    match log::set_logger(&*LOGGER).map(|()| log::set_max_level(LevelFilter::Info)) {
        Ok(()) => Ok(CH.0.clone()),
        Err(e) => Err(e),
    }
}

pub fn subscribe() -> tokio::sync::broadcast::Receiver<String> {
    LOGGER.tx.subscribe()
}
