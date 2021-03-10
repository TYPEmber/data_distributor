use socket2::{Domain, SockAddr, Socket, Type};

use log::{debug, error, info, trace, warn};
use std::sync::{Arc, Mutex};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}

mod logger;

pub async fn initial() -> tokio::sync::broadcast::Sender<()> {
    crate::logger::init();

    let addrs = vec![
        "127.0.0.1:19208".parse().unwrap(),
        "127.0.0.1:19210".parse().unwrap(),
        //"192.168.200.3:19209".parse().unwrap(),
        // "192.168.200.3:19210".parse().unwrap(),
        //"10.0.0.1:19209".parse().unwrap(),
    ];

    info!("{:?}", addrs);

    let addrs_2 = vec![
        "127.0.0.1:19211".parse().unwrap(),
        "127.0.0.1:19212".parse().unwrap(),
        //"192.168.200.3:19201".parse().unwrap(),
        // "192.168.200.3:19210".parse().unwrap(),
        //"10.0.0.1:19209".parse().unwrap(),
    ];

    // let addrs = Arc::new(addrs);
    // let socket = Arc::new(socket);
    // let sender = Arc::new(sender);
    // let mut buf = Box::new([0u8; 65535]);
    // let mut count = 0usize;

    let (tx, mut rx) = tokio::sync::broadcast::channel(1);

    let mut dis_obj = Distributor::new("127.0.0.1:5503".parse().unwrap(), addrs, tx.clone());
    let mut dis_obj_2 = Distributor::new("127.0.0.1:19210".parse().unwrap(), addrs_2, tx.clone());
    let mut dis_vec = vec![dis_obj, dis_obj_2];
    let map = generate_sender_map(&mut dis_vec, tx.clone());

    for dis_obj in dis_vec.drain(..) {
        dis_obj.run(map.clone()).await;
    }

    tx
}

pub fn generate_socket(
    bind_addr: std::net::SocketAddr,
    recv_buff_size: usize,
    send_buff_size: usize,
) -> tokio::net::UdpSocket {
    let sender = Socket::new(Domain::ipv4(), Type::dgram(), None).unwrap();
    sender.bind(&SockAddr::from(bind_addr)).unwrap();
    sender.set_nonblocking(true).unwrap();
    sender.set_recv_buffer_size(recv_buff_size).unwrap();
    sender.set_send_buffer_size(send_buff_size).unwrap();
    let sender = sender.into_udp_socket();
    tokio::net::UdpSocket::from_std(sender).unwrap()
}
use std::collections::{HashMap, HashSet};

pub struct Distributor {
    pub receiver: tokio::net::UdpSocket,
    pub recv_speed: Arc<AtomicUsize>,
    pub recv_speed_acc: Arc<AtomicUsize>,
    pub stop_broadcast_sender: tokio::sync::broadcast::Sender<()>,
    pub remote: Vec<Arc<RemoteInfo>>,
}

impl Distributor {
    pub fn new(
        listen_addr: std::net::SocketAddr,
        remote_addrs: Vec<std::net::SocketAddr>,
        stop_broadcast_sender: tokio::sync::broadcast::Sender<()>,
    ) -> Self {
        Self {
            receiver: generate_socket(listen_addr, 1024 * 1024, 1024),
            recv_speed: Arc::new(AtomicUsize::new(0)),
            recv_speed_acc: Arc::new(AtomicUsize::new(0)),
            remote: remote_addrs
                .iter()
                .map(|addr| {
                    Arc::new(RemoteInfo {
                        addr: *addr,
                        speed: AtomicUsize::new(0),
                        speed_acc: AtomicUsize::new(0),
                    })
                })
                .collect(),
            stop_broadcast_sender: stop_broadcast_sender,
        }
    }

    pub async fn run(
        mut self,
        sender_map: Arc<HashMap<std::net::SocketAddr, crossbeam::channel::Sender<SendRequest>>>,
    ) {
        let mut stop_broadcast_recv = self.stop_broadcast_sender.subscribe();

        let recv_speed = self.recv_speed.clone();
        let recv_speed_acc = self.recv_speed_acc.clone();

        let local_addr = self.receiver.local_addr().unwrap();

        let remotes = self.remote.clone();
        let mut buf = Box::new([0u8; 65536]);

        // receive thread
        tokio::spawn(async move {
            tokio::select! {
                _ = async  {
                        loop {
                        if let Ok((len, _)) = self.receiver.recv_from(&mut buf[..]).await{
                            self.recv_speed_acc.fetch_add(len, Ordering::SeqCst);
                            let data = Arc::new(buf[..len].to_vec());
                            for remote in &self.remote {
                                if let Err(e) = sender_map
                                    .get(&remote.addr)
                                    .unwrap()
                                    .send(SendRequest::new(remote.clone(), data.clone()))
                                    {
                                        error!("[{}]", e);
                                    }
                            }
                        }

                }} => { },

                 // caculate speed
            _ = async  {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(1_000));
                loop{
                    interval.tick().await;
                    recv_speed.store(recv_speed_acc.load(Ordering::SeqCst), Ordering::SeqCst);
                    recv_speed_acc.store(0, Ordering::SeqCst);
                    info!("[SPEED][{}][IN] {} bps", local_addr, 8 * recv_speed.load(Ordering::SeqCst));
                    for remote in &remotes {
                        remote
                            .speed
                            .store(remote.speed_acc.load(Ordering::SeqCst), Ordering::SeqCst);
                        remote.speed_acc.store(0, Ordering::SeqCst);
                        info!(
                            "[SPEED][{}][OUT] {} bps",
                            remote.addr,
                            8 * remote.speed.load(Ordering::SeqCst)
                        );
                    }
                }
            } => {},
            _ = stop_broadcast_recv.recv() => { warn!("[CLOSED][{}]", local_addr) },
            };
        });
    }
}

pub fn generate_sender_map(
    distributors: &mut Vec<Distributor>,
    stop_broadcast_sender: tokio::sync::broadcast::Sender<()>,
) -> Arc<HashMap<std::net::SocketAddr, crossbeam::channel::Sender<SendRequest>>> {
    let mut map = HashMap::new();
    let mut local_ips = HashMap::new();
    // It will choose the same sender port for every net card
    let mut socket = std::net::UdpSocket::bind("0.0.0.0:0").unwrap();
    for item in distributors {
        for remote in &mut item.remote {
            socket.connect(remote.addr).unwrap();
            let local = socket.local_addr().unwrap();

            if !local_ips.contains_key(&local) {
                local_ips.insert(local, generate_sender_thread(stop_broadcast_sender.clone()));
            }
            map.insert(remote.addr, local_ips.get(&local).unwrap().clone());
        }
    }

    //println!("{:?}", local_ips);

    Arc::new(map)
}

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub struct RemoteInfo {
    pub addr: std::net::SocketAddr,
    pub speed: AtomicUsize,
    pub speed_acc: AtomicUsize,
}

pub struct SendRequest {
    pub remote: Arc<RemoteInfo>,
    pub data: Arc<Vec<u8>>,
}
impl SendRequest {
    pub fn new(remote: Arc<RemoteInfo>, data: Arc<Vec<u8>>) -> Self {
        Self { remote, data }
    }
}

pub fn generate_sender_thread(
    mut stop_broadcast_sender: tokio::sync::broadcast::Sender<()>,
) -> crossbeam::channel::Sender<SendRequest> {
    let (tx, rx) = crossbeam::channel::unbounded::<SendRequest>();
    let socket = generate_socket("0.0.0.0:0".parse().unwrap(), 1024, 3 * 1024 * 1024);

    let rrx = rx.clone();

    let mut stop_for_send_thread = stop_broadcast_sender.subscribe();

    tokio::spawn(async move {
        tokio::select! {
            _ = async {
                loop {
                if let Ok(req) = rx.recv() {
                    let len = socket
                        .send_to(&req.data[..], req.remote.addr)
                        .await
                        .unwrap();
                    req.remote.speed_acc.fetch_add(len, Ordering::SeqCst);
                }
                else{
                    break;
                }
            }
            }=>{},
            _ = async{
                let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(500));

                let mut flag_count = 0usize;
                interval.tick().await;
                interval.tick().await;
                loop {
                    let count_last = rrx.len();
                    interval.tick().await;
                    let count = rrx.len();
                    if count > count_last {
                        if flag_count > 3 {
                            println!("buffer len: {}", count);
                            for i in 0..count / 2 {
                                rrx.try_recv();
                            }
                            flag_count = 4;
                        }
                        flag_count += 1;
                    } else {
                        flag_count = 0;
                    }
                }
            } => {},
            _ = stop_for_send_thread.recv() =>{}
        }
    });

    tx
}
