use log::{Level, Metadata, Record};
use std::sync::Arc;

struct AsyncLogger {
    tx: tokio::sync::broadcast::Sender<String>,
}

impl AsyncLogger {
    pub fn new() -> Self {
        let (tx, _) = tokio::sync::broadcast::channel::<String>(20);

        let mut rx0 = tx.subscribe();
        tokio::spawn(async move {
            loop {
                println!("{}", rx0.recv().await.unwrap_or("slow".to_string()));
            }
        });
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
                    format!("[{}]{}", record.level(), record.args())
                })
                .unwrap();
        }
    }

    fn flush(&self) {}
}

use log::{LevelFilter, SetLoggerError};

lazy_static::lazy_static! {
    static  ref LOGGER: AsyncLogger = AsyncLogger::new();
}

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&*LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
}
