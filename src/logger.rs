use log::{debug, error, info, trace, warn};
use log::{Level, Metadata, Record};

/// 该结构体包含了一个广播的通道发送端，所需要保存到logger
/// 的信息，通过该通道发送。
struct AsyncLogger {
    tx: tokio::sync::broadcast::Sender<String>,
}

impl AsyncLogger {
    /// 生成一个实例
    pub fn new() -> Self {
        let (tx, _) = tokio::sync::broadcast::channel::<String>(20);

        AsyncLogger { tx }
    }
}

impl log::Log for AsyncLogger {
    /// 设置所需要保存的日志信息级别
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }
    /// 格式化不同级别日志所需要包括的信息，然后在通道发送端发送
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
    /// 刷新日志缓存
    fn flush(&self) {}
}

use log::{LevelFilter, SetLoggerError};

lazy_static::lazy_static! {
    static ref LOGGER: AsyncLogger = AsyncLogger::new();
}
/// 初始化LOGGER实例，设置其要保存的日志信息级别
pub fn init() {
    log::set_logger(&*LOGGER)
        .map(|()| log::set_max_level(LevelFilter::Info))
        .unwrap();
}

/// 该函数返回广播通道的接收端，用于接收日志信息
pub fn subscribe() -> tokio::sync::broadcast::Receiver<String> {
    LOGGER.tx.subscribe()
}
