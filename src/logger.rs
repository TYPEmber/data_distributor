use log::{Level, Metadata, Record};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

struct AsyncLogger {
    tx: tokio::sync::broadcast::Sender<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct SpeedRequest {
    vec: Vec<std::net::SocketAddr>,
}

use warp::Filter;
impl AsyncLogger {
    pub fn new() -> Self {
        let (tx, _) = tokio::sync::broadcast::channel::<String>(20);

        let map = Arc::new(std::sync::Mutex::new(std::collections::HashMap::<
            String,
            String,
        >::new()));
        let map_0 = map.clone();

        let pattern = Regex::new(r"[\[](.*?)[\]]").unwrap();

        let mut rx0 = tx.subscribe();
        tokio::spawn(async move {
            loop {
                let s = rx0.recv().await.unwrap_or("slow".to_string());
                println!("{}", s);
                let mut res = pattern.find_iter(&s);
                if let Some(v) = res.next() {
                    if v.as_str() == "[INFO]" {
                        if let Some(v) = res.next() {
                            if v.as_str() == "[SPEED]" {
                                let addr = res.next().unwrap().as_str().to_string();
                                let speed = res.next().unwrap().as_str().to_string();

                                let addr = addr[1..(addr.len() - 1)].to_string();
                                let speed = speed[1..(speed.len() - 1)].to_string();
                                map.lock().unwrap().insert(addr, speed);
                            }
                        }
                    }
                }
            }
        });

        // // Match any request and return hello world!
        // let routes = warp::any().map(move || {
        //     format!(
        //         "127.0.0.1:5503 - {} bps",
        //         map_0
        //             .lock()
        //             .unwrap()
        //             .get("[127.0.0.1:5503]")
        //             .unwrap_or(&"NULL".to_string())
        //     )
        // });

        let main_page = warp::path::end().and(warp::fs::file(
            "E:/Projects/rust/data_distributor/src/ui.html",
        ));
        let other_page = warp::path("test").and(warp::fs::file(
            "E:/Projects/rust/data_distributor/src/ui copy.html",
        ));
        let get_speed = warp::post()
            .and(warp::path("api"))
            .and(warp::path("speed"))
            // .and(warp::body::bytes())
            // .map(|p| {
            //     let sstr = serde_json::to_string(&SpeedRequest {
            //         vec: vec![
            //             "127.0.0.1:5503".parse().unwrap(),
            //             "127.0.0.1:19208".parse().unwrap(),
            //         ],
            //     }).unwrap();
            //     println!("{}", &sstr);
            //     println!("{:?}", p);
            //     "got a stream"
            // });
            .and(warp::body::json())
            .map(move |p: SpeedRequest| {
                println!("asdfadfadsf {:?}", p);
                let map_locked = map_0.lock().unwrap();
                let res: Vec<String> = p
                    .vec
                    .iter()
                    .map(|addr| {
                        map_locked
                            .get(&addr.to_string())
                            .unwrap_or(&"0".to_string())
                            .to_string()
                    })
                    .collect();
                warp::http::Response::builder().body((serde_json::to_string(&res).unwrap()))
            });
        tokio::spawn(async move {
            warp::serve(main_page.or(get_speed).or(other_page))
                .run(([127, 0, 0, 1], 8080))
                .await;
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
