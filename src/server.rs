use log::{debug, error, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use warp::{
    http::{Response, StatusCode},
    Filter,
};

#[derive(Debug, Deserialize, Serialize, Clone)]
struct SpeedRequest {
    vec: Vec<String>,
}

pub fn run(
    mut msg_rx: tokio::sync::broadcast::Receiver<String>,
    stop_tx: tokio::sync::broadcast::Sender<()>,
) -> Result<u16, String> {
    let map = Arc::new(std::sync::Mutex::new(std::collections::HashMap::<
        String,
        String,
    >::new()));
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

                                    map.lock().unwrap().insert(in_out, speed);
                                }
                                "GROUP" => {
                                    //println!("{} {}",s.find("GROUP").unwrap(), s[s.find("GROUP").unwrap() + 5 + 1..].to_string());
                                    let json = s[s.find(v).unwrap() + v.len() + 1..].to_owned();

                                    map.lock().unwrap().insert(v.to_owned(), json);
                                }
                                "CLOSED" => loop {
                                    if let Some(addr) = res.next() {
                                        map.lock().unwrap().remove(addr);
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
                    let mut try_count = 0;
                    while stop_tx_m.receiver_count() > 1 {
                        try_count += 1;
                        println!("{}", stop_tx_m.receiver_count());
                        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                        if try_count > 10 {
                            return Err(warp::reject::not_found());
                        }
                    }

                    Ok("stop success!".to_string())
                } else {
                    Err(warp::reject::not_found())
                }
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
                    .body("RUNNING!")
            } else {
                if let Ok(group) = crate::params::Group::load("params.json") {
                    let dis_vec = group.get_flat_enable();
                    match crate::initial(dis_vec, 1024 * 1024, 1024 * 1024, stop_tx_m.clone()) {
                        Ok((dis_vec, sender_map)) => {
                            tokio::spawn(async move {
                                crate::run(dis_vec, sender_map).await;
                            });
                            warp::http::Response::builder().body("start")
                        }
                        Err(e) => warp::http::Response::builder().body("start failed"),
                    }
                } else {
                    warp::http::Response::builder().body("save failed")
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
            match crate::initial(dis_vec, 1024 * 1024, 1024 * 1024, stop_tx_m.clone()) {
                Ok((dis_vec, sender_map)) => {
                    if let Ok(json) = group.get_json() {
                        info!("GROUP {}", json);
                        if let Ok(()) = group.save("params.json") {
                            tokio::spawn(async move {
                                crate::run(dis_vec, sender_map).await;
                            });
                            warp::http::Response::builder().body("start")
                        } else {
                            warp::http::Response::builder().body("save failed")
                        }
                    } else {
                        warp::http::Response::builder().body("serialize failed")
                    }
                }
                Err(e) => warp::http::Response::builder().body("start failed"),
            }
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
                        .get(addr)
                        .unwrap_or(&"0 0".to_string())
                        .to_string()
                })
                .collect();
            warp::http::Response::builder()
                .header("content-type", "text/html; charset=utf-8")
                .header("cache-control", "no-cache")
                .header("x-content-type-options", "nosniff")
                .body((serde_json::to_string(&res).unwrap()))
        });

    tokio::spawn(async move {
        warp::serve(
            main_page
                // .or(basic_get)
                .or(get_speed)
                .or(get_group)
                .or(stop)
                .or(start)
                .or(start_save),
        )
        .run(([0, 0, 0, 0], 8080))
        .await;
    });

    Ok(8080u16)
}
