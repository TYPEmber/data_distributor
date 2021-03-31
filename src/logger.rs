use log::{Level, Metadata, Record};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use warp::{
    http::{Response, StatusCode},
    Filter,
};

struct AsyncLogger {
    tx: tokio::sync::broadcast::Sender<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct SpeedRequest {
    vec: Vec<std::net::SocketAddr>,
}

impl AsyncLogger {
    pub fn new() -> Self {
        let (tx, _) = tokio::sync::broadcast::channel::<String>(20);

        let map = Arc::new(std::sync::Mutex::new(std::collections::HashMap::<
            String,
            String,
        >::new()));
        let map_0 = map.clone();
        let map_1 = map.clone();

        let mut rx0 = tx.subscribe();
        tokio::spawn(async move {
            loop {
                let s = rx0.recv().await.unwrap_or("slow".to_string());
                println!("{}", s);
                let mut res = s.split(" ");
                if let Some(v) = res.next() {
                    match v {
                        "[INFO]" => {
                            if let Some(v) = res.next() {
                                match v {
                                    "SPEED" => {
                                        res.next();
                                        let addr = res.next().unwrap().to_string();
                                        let speed = res.next().unwrap().to_string();

                                        map.lock().unwrap().insert(addr, speed);
                                    }
                                    "GROUP" => {
                                        //println!("{} {}",s.find("GROUP").unwrap(), s[s.find("GROUP").unwrap() + 5 + 1..].to_string());
                                        let json =
                                            s[s.find(v).unwrap() + v.len() + 1..].to_string();

                                        map.lock().unwrap().insert(v.to_string(), json);
                                    }
                                    &_ => {}
                                }
                            }
                        }
                        &_ => {}
                    }
                }
            }
        });

        //println!("{:?}",  std::env::current_dir().unwrap_or_default());

        let main_page = warp::path::end().and(warp::fs::file("ui.html"));

        let stop = warp::post()
            .and(warp::path("api"))
            .and(warp::path("ctrl"))
            .and(warp::path("stop"))
            .and_then(|| async move {
                if let Ok(count) = CH.0.send(()) {
                    // TODO: 尝试超时返回失败
                    while CH.0.receiver_count() > 1 {
                        println!("{}", CH.0.receiver_count());
                        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                    }
                    Ok("stop success!".to_string())
                } else {
                    Err(warp::reject::not_found())
                }
            });

        let start = warp::post()
            .and(warp::path("api"))
            .and(warp::path("ctrl"))
            .and(warp::path("start"))
            .map(|| {
                if CH.0.receiver_count() > 1 {
                    Response::builder()
                        .status(StatusCode::BAD_REQUEST)
                        .body("RUNNING!")
                } else {
                    let group = crate::params::Group::load("params.json");
                    let dis_vec = group.get_flat_enable();
                    match crate::initial(dis_vec, 1024 * 1024, 1024 * 1024, CH.0.clone()) {
                        Ok((dis_vec, sender_map)) => {
                            tokio::spawn(async move {
                                crate::run(dis_vec, sender_map).await;
                            });
                            warp::http::Response::builder().body("start")
                        }
                        Err(e) => warp::http::Response::builder().body("start failed"),
                    }
                }
            });

        let set_group = warp::post()
            .and(warp::path("api"))
            .and(warp::path("group"))
            .and(warp::path("set"))
            .and(warp::body::json())
            .map(|group: crate::params::Group| {
                group.save("params.json");
            });

        let get_group = warp::post()
            .and(warp::path("api"))
            .and(warp::path("group"))
            .and(warp::path("get"))
            .map(move || {
                map_1
                    .lock()
                    .unwrap()
                    .get(&"GROUP".to_string())
                    // unprepared
                    .unwrap_or(&"".to_string())
                    .to_string()
            });

        let get_speed = warp::post()
            .and(warp::path("api"))
            .and(warp::path("speed"))
            .and(warp::path("get"))
            .and(warp::body::json())
            .map(move |p: SpeedRequest| {
                //println!("asdfadfadsf {:?}", p);
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
            warp::serve(main_page.or(get_speed).or(get_group).or(stop).or(start))
                .run(([0, 0, 0, 0], 8080))
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
