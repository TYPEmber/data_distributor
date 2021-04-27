use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use dashmap::DashMap;
use warp::{
    http::{Response, StatusCode},
    Filter,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct SpeedRequest {
    vec: Vec<String>,
}
/// 运行服务器端线程
/// 
/// 该服务器端线程，主要实现了五个类别的HTTP请求响应。
/// # start
/// 在Web客户端启动程序。如果软件在之前已经启动，会反馈一个running信息，
/// 如果未启动，则成功运行后，会反馈一个start信息。
/// # start_save
/// 该请求，会在启动软件同时，把在Web客户端设置的配置信息，已json文件形式
/// 保存到本地。
/// # stop
/// 客户端发送stop的HTTP请求后，软件会停止运行。
/// # get_speed
/// 收到该HTTP请求，会在HTTP响应中包含数据包的接收，发送速率
/// # get_group 
/// 收到该HTTP请求，会在响应中包含当前软件运行的配置信息
pub async fn run(
    listen_port: u16,
    mut msg_rx: tokio::sync::broadcast::Receiver<String>,
    stop_tx: tokio::sync::broadcast::Sender<()>,
) {
   /* let map = Arc::new(std::sync::Mutex::new(std::collections::HashMap::<
        String,
        String,
    >::new())); */
    let map =Arc::new(DashMap::<String, String>::new());
    let map_0 = map.clone();
    let map_1 = map.clone();
    let map_2 = map.clone();

    tokio::spawn(async move {
        loop {
            let s = msg_rx.recv().await.unwrap_or("slow".to_string());
            println!("{}", s);
            let mut res = s.split(" ");
            if let Some(v) = res.next() {
                match v {
                    "[INFO]" => {
                        if let Some(v) = res.next() {
                            match v {
                                "SPEED" => {
                                    let mut in_out = res.next().unwrap().to_owned();
                                    let addr = res.next().unwrap();
                                    in_out.push_str("_");
                                    in_out.push_str(addr);
                                    let mut speed = res.next().unwrap().to_owned();
                                    let pkg_speed = res.next().unwrap();
                                    speed.push_str(" ");
                                    speed.push_str(pkg_speed);
                                    map.insert(in_out,speed);
                                   // map.lock().unwrap().insert(in_out, speed);
                                }
                                "GROUP" => {
                                    //println!("{} {}",s.find("GROUP").unwrap(), s[s.find("GROUP").unwrap() + 5 + 1..].to_string());
                                    let json = s[s.find(v).unwrap() + v.len() + 1..].to_owned();
                                      map.insert(v.to_owned(),json);
                                     //map.lock().unwrap().insert(v.to_owned(), json);
                                }
                                "CLOSED" => loop {
                                    if let Some(addr) = res.next() {
                                       map.remove(addr);
                                    //    map.lock().unwrap().remove(addr);
                                    } else {
                                        break;
                                    }
                                },
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

    let main_page = warp::get().and(warp::fs::dir("dd_gui/dist/"));
    //let main_page = warp::get().and(warp::fs::file("ui.html"));

    let stop_tx_m = stop_tx.clone();
    let stop = warp::post()
        .and(warp::path("api"))
        .and(warp::path("ctrl"))
        .and(warp::path("stop"))
        .and_then(move || {
            // capture
            let stop_tx_m = stop_tx_m.clone();
            async move {
                if let Ok(_) = stop_tx_m.send(()) {
                    // TODO: 尝试超时返回失败
                    let mut try_count = 0usize;
                    while stop_tx_m.receiver_count() > 1 {
                        try_count += 1;
                        println!("{}", stop_tx_m.receiver_count());
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        if try_count > 10 {
                            return Ok("stop failed");
                        }
                    }

                    return Ok("stop success");
                } else {
                    return Ok("has stopped");
                }
                Err(warp::reject::not_found())
            }
        });

    let stop_tx_m = stop_tx.clone();
    let start = warp::post()
        .and(warp::path("api"))
        .and(warp::path("ctrl"))
        .and(warp::path("start"))
        .map(move || {
            if stop_tx_m.receiver_count() > 1 {
                Response::builder()
                    .status(StatusCode::BAD_REQUEST)
                    .body("RUNNING!".to_owned())
            } else {
                if let Ok(group) = crate::params::Group::load("params.json") {
                    let dis_vec = group.get_flat_enable();
                    match crate::initial(dis_vec, group.send_buffer, stop_tx_m.clone()) {
                        Ok((dis_vec, sender_map)) => {
                            tokio::spawn(async move {
                                crate::run(dis_vec, sender_map).await;
                            });
                            warp::http::Response::builder().body("start".to_owned())
                        }
                        Err(e) => {
                            warp::http::Response::builder().body(format!("{}", e))
                        }
                    }
                } else {
                    warp::http::Response::builder().body("save failed".to_owned())
                }
            }
        });

    let stop_tx_m = stop_tx.clone();
    let start_save = warp::post()
        .and(warp::path("api"))
        .and(warp::path("ctrl"))
        .and(warp::path("start_save"))
        .and(warp::body::json())
        .map(move |group: crate::params::Group| {
            println!("{:?}", group);
            let dis_vec = group.get_flat_enable();
            match crate::initial(dis_vec, group.send_buffer, stop_tx_m.clone()) {
                Ok((dis_vec, sender_map)) => {
                    if let Ok(json) = group.get_json() {
                        info!("GROUP {}", json);
                        if let Ok(()) = group.save("params.json") {
                            tokio::spawn(async move {
                                crate::run(dis_vec, sender_map).await;
                            });
                            warp::http::Response::builder().body("start success".to_owned())
                        } else {
                            warp::http::Response::builder().body("save failed".to_owned())
                        }
                    } else {
                        warp::http::Response::builder().body("serialize failed".to_owned())
                    }
                }
                Err(e) => warp::http::Response::builder().body(format!("{}", e)),
            }
        });

        let get_group = warp::post()
        .and(warp::path("api"))
        .and(warp::path("group"))
        .and(warp::path("get"))
        .map(move || {
            map_1
               // .lock()
               // .unwrap()
                .get("GROUP".into())
                // unprepared
                .unwrap()
                .to_owned()
        });


        let get_speed = warp::post()
        .and(warp::path("api"))
        .and(warp::path("speed"))
        .and(warp::path("get"))
        .and(warp::body::json())
        .map(move |p: SpeedRequest| {
            //println!("asdfadfadsf {:?}", p);
           // let map_locked = map_0.lock().unwrap();
            let res: Vec<String> = p
                .vec
                .iter()
                .map(|addr| {
                    map_0
                        .get(addr)
                        .unwrap()
                        .to_string()
                })
                .collect();
            warp::http::Response::builder()
                .header("content-type", "text/html; charset=utf-8")
                .header("cache-control", "no-cache")
                .header("x-content-type-options", "nosniff")
                .body(serde_json::to_string(&res).unwrap())
        });

    warp::serve(
        main_page
            // .or(basic_get)
            .or(get_speed)
            .or(get_group)
            .or(stop)
            .or(start)
            .or(start_save),
    )
    .run(([0, 0, 0, 0], listen_port))
    .await;
}
