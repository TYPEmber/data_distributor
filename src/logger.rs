use log::{Level, Metadata, Record};

struct AsyncLogger;

impl log::Log for AsyncLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        //println!("{:?}", record);
        if self.enabled(record.metadata()) {
            if record.level() == Level::Error {
                println!(
                    "[{}]{}[{}][{}][{}]",
                    record.level(),
                    record.args(),
                    record.module_path().unwrap_or("<unamed>"),
                    record.file().unwrap_or("<unamed>"),
                    record.line().unwrap_or(0)
                );
            } else {
                println!("[{}]{}", record.level(), record.args());
            }
        }
    }

    fn flush(&self) {}
}

use log::{LevelFilter, SetLoggerError};

static LOGGER: AsyncLogger = AsyncLogger;

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
}
