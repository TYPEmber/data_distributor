use socket2::{Domain, SockAddr, Socket, Type};

use log::{debug, error, info, trace, warn};
use std::sync::{Arc, Mutex};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}

pub mod logger;
pub mod params;

pub fn initial(
    distributors: Vec<(std::net::SocketAddr, Vec<std::net::SocketAddr>)>,
    recv_buff_size: usize,
    send_buff_size: usize,
    stop_trigger: tokio::sync::broadcast::Sender<()>,
) -> Result<
    (
        Vec<Distributor>,
        Arc<HashMap<std::net::SocketAddr, crossbeam::channel::Sender<SendRequest>>>,
    ),
    std::io::Error,
> {
    let mut dis_vec = vec![];

    for (local_addr, remote_addrs) in distributors.into_iter() {
        dis_vec.push(Distributor::new(
            local_addr,
            remote_addrs,
            stop_trigger.clone(),
        )?);
    }

    let dis_vec = dis_vec;

    let map = generate_sender_map(&dis_vec, stop_trigger.clone())?;

    Ok((dis_vec, map))
}

pub async fn run(
    dis_vec: Vec<Distributor>,
    map: Arc<HashMap<std::net::SocketAddr, crossbeam::channel::Sender<SendRequest>>>,
) {
    for dis_obj in dis_vec.into_iter() {
        dis_obj.run(map.clone()).await;
    }
}

pub fn generate_socket(
    bind_addr: std::net::SocketAddr,
    recv_buff_size: usize,
    send_buff_size: usize,
) -> Result<tokio::net::UdpSocket, std::io::Error> {
    let sender = Socket::new(Domain::ipv4(), Type::dgram(), None)?;
    sender.bind(&SockAddr::from(bind_addr))?;
    sender.set_nonblocking(true).unwrap();
    sender.set_recv_buffer_size(recv_buff_size)?;
    sender.set_send_buffer_size(send_buff_size)?;
    let sender = sender.into_udp_socket();
    tokio::net::UdpSocket::from_std(sender)
}
use std::collections::{HashMap, HashSet};

pub struct Distributor {
    pub receiver: tokio::net::UdpSocket,
    pub recv_speed: Arc<AtomicUsize>,
    pub recv_speed_acc: Arc<AtomicUsize>,
    pub recv_package: Arc<AtomicUsize>,
    pub recv_package_acc: Arc<AtomicUsize>,
    pub stop_broadcast_sender: tokio::sync::broadcast::Sender<()>,
    pub remote: Vec<Arc<RemoteInfo>>,
}

impl Distributor {
    pub fn new(
        listen_addr: std::net::SocketAddr,
        remote_addrs: Vec<std::net::SocketAddr>,
        stop_broadcast_sender: tokio::sync::broadcast::Sender<()>,
    ) -> Result<Self, std::io::Error> {
        Ok(Self {
            receiver: generate_socket(listen_addr, 65 * 1024 * 1024, 1024)?, // 大十五倍
            recv_speed: Arc::new(AtomicUsize::new(0)),
            recv_speed_acc: Arc::new(AtomicUsize::new(0)),
            recv_package: Arc::new(AtomicUsize::new(0)),
            recv_package_acc: Arc::new(AtomicUsize::new(0)),
            remote: remote_addrs
                .iter()
                .map(|addr| {
                    Arc::new(RemoteInfo {
                        addr: *addr,
                        speed: AtomicUsize::new(0),
                        speed_acc: AtomicUsize::new(0),
                        package: AtomicUsize::new(0),
                        package_acc: AtomicUsize::new(0),
                    })
                })
                .collect(),
            stop_broadcast_sender: stop_broadcast_sender,
        })
    }

    pub async fn run(
        mut self,
        sender_map: Arc<HashMap<std::net::SocketAddr, crossbeam::channel::Sender<SendRequest>>>,
    ) {
        let mut stop_broadcast_recv = self.stop_broadcast_sender.subscribe();

        let recv_speed = self.recv_speed.clone();
        let recv_speed_acc = self.recv_speed_acc.clone();
        let recv_package = self.recv_package.clone();
        let recv_package_acc = self.recv_package_acc.clone();
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
                             self.recv_package_acc.fetch_add(1,Ordering::SeqCst);
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
                     recv_package.store(recv_package_acc.load(Ordering::SeqCst), Ordering::SeqCst);
                     recv_package_acc.store(0, Ordering::SeqCst);
                     info!("PACKAGES IN {} {}  pps", local_addr, recv_package.load(Ordering::SeqCst));
                     info!("SPEED IN {} {}  bps", local_addr, 8 * recv_speed.load(Ordering::SeqCst));
                     for remote in &remotes {
                         remote
                             .speed
                             .store(remote.speed_acc.load(Ordering::SeqCst), Ordering::SeqCst);
                         remote.speed_acc.store(0, Ordering::SeqCst);
                         remote.package
                             .store(remote.package_acc.load(Ordering::SeqCst), Ordering::SeqCst);
                         remote.package_acc.store(0, Ordering::SeqCst);
                         info!(
                             "PACKAGES OUT {} {} pps",
                             remote.addr,
                             remote.package.load(Ordering::SeqCst)
                         );
                         info!(
                             "SPEED OUT {} {} bps",
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
    distributors: &Vec<Distributor>,
    stop_broadcast_sender: tokio::sync::broadcast::Sender<()>,
) -> Result<
    Arc<HashMap<std::net::SocketAddr, crossbeam::channel::Sender<SendRequest>>>,
    std::io::Error,
> {
    let mut map = HashMap::new();
    let mut local_ips = HashMap::new();
    // It will choose the same sender port for every net card
    let mut socket = std::net::UdpSocket::bind("0.0.0.0:0")?;
    for item in distributors {
        for remote in &item.remote {
            socket.connect(remote.addr)?;
            let local = socket.local_addr()?;

            if !local_ips.contains_key(&local) {
                local_ips.insert(
                    local,
                    generate_sender_thread(stop_broadcast_sender.clone())?,
                );
            }
            map.insert(remote.addr, local_ips.get(&local).unwrap().clone());
        }
    }

    //println!("{:?}", local_ips);

    Ok(Arc::new(map))
}

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub struct RemoteInfo {
    pub addr: std::net::SocketAddr,
    pub speed: AtomicUsize,
    pub speed_acc: AtomicUsize,
    pub package: AtomicUsize,
    pub package_acc: AtomicUsize,
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
    stop_broadcast_sender: tokio::sync::broadcast::Sender<()>,
) -> Result<crossbeam::channel::Sender<SendRequest>, std::io::Error> {
    let (tx, rx) = crossbeam::channel::unbounded::<SendRequest>();
    let socket = generate_socket("0.0.0.0:0".parse().unwrap(), 1024, 15 * 1024 * 1024)?; // 发送的buffer增长5倍

    let rrx = rx.clone();

    let mut stop_for_send_thread = stop_broadcast_sender.subscribe();
    let mut stop_for_send_thread_1 = stop_broadcast_sender.subscribe();
    let mut send_size = Arc::new(AtomicUsize::new(0));
    let mut send_size1 = send_size.clone();
    tokio::spawn(async move {
        tokio::select! {
         /*   _ = async {
                let mut interval =  tokio::time::interval(tokio::time::Duration::from_millis(1));
                loop {
                    //tokio::task::yield_now().await;
                   match rx.try_recv() {
                       Ok(req)=>{
                            match socket
                            .send_to(&req.data[..], req.remote.addr)
                            .await{
                                Ok(len)=>{
                                    req.remote.speed_acc.fetch_add(len, Ordering::SeqCst);
                                    req.remote.package_acc.fetch_add(1, Ordering::SeqCst);
                                    send_size.fetch_add(1, Ordering::SeqCst);
                                }
                                Err(e)=>{
                                    warn!("SEND_TO {} {}", req.remote.addr, e);
                                    // 可能是虚拟网卡被移除
                                    // 因此间隔尝试
                                    tokio::time::sleep(tokio::time::Duration::from_millis(30_000)).await;
                                }
                            }
                        },
                        Err(e)=>{
                            if e == crossbeam::channel::TryRecvError::Empty{
                                //tokio::task::yield_now();
                                interval.tick().await;
                            }
                            else{
                                break;
                            }
                        }
                    }
                }
                          }=>{println!("sfadfasdf")},*/
            _ = async {
                loop {
                    //tokio::task::yield_now().await;
                    if let Ok(req) = rx.recv() {
                        match socket
                            .send_to(&req.data[..], req.remote.addr)
                            .await{
                                Ok(len)=>{req.remote.speed_acc.fetch_add(len, Ordering::SeqCst);
                                          req.remote.package_acc.fetch_add(1, Ordering::SeqCst);
                                          send_size.fetch_add(1, Ordering::SeqCst);

                                }
                                Err(e)=>{
                                    warn!("SEND_TO {} {}", req.remote.addr, e);
                                    // 可能是虚拟网卡被移除
                                    // 因此间隔尝试
                                    tokio::time::sleep(tokio::time::Duration::from_millis(30_000)).await;
                                }
                            }
                      }
                    else {
                        break;
                    }
                }
            }=>{ },
            _ =tokio::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(1000));

                interval.tick().await;
                interval.tick().await;
                let mut flag_count = 0usize;
                //tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                loop {



                    if let Ok(()) = stop_for_send_thread_1.try_recv(){
                        break;
                    }

                   /* let count_last = rrx.len();
                    interval.tick().await;
                    let count = rrx.len();
                    if count > count_last {
                        if flag_count > 3 {
                            println!("buffer len: {}", count);
                            for _ in 0..count / 2 {
                                rrx.try_recv();
                            }
                            flag_count = 4;
                        }
                        flag_count += 1;
                    } else {
                        flag_count = 0;
                    }*/
                    let size_last= send_size1.load(Ordering::SeqCst); // 第一次获取发送的包数量
                     let count_last= rrx.len();
                       interval.tick().await;
                       let count = rrx.len();
                       let size= send_size1.load(Ordering::SeqCst);  // 第二次获取网卡发送的包数量
                       let send_pack=size - size_last;  // 在interval时间内发送的包数量
                     //   println!("count is {},and size is {}",count,send_pack);                                        // 采样了限幅消抖滤波法
                      if count>send_pack{                     // count为当前采样间隔，包发送以后，channel剩余的包数量
                       println!("buffer len: {}/////////////////////////////////", count);       // count大于发送速率主动丢包，确定了缓存延迟的上限，大于等于软件设置的采样时间间隔
                        for i in 0..count/2{
                            rrx.try_recv();
                        }
                        flag_count=0;
                       }else{
                       if count > count_last {              // 上面限幅，这里继续用消抖，目的是在确定延迟上限情况下，可以有条件触发丢包，来优化延迟
                           if flag_count > 3 {              // 因为有丢包上限触发机制，这里大于5，触发条件可以稍微宽松一点，减少丢包频率
                               println!("buffer len2: {}/////////////////", count);
                               for i in 0..count/3  {        // 当触发时，丢包量减少一些
                                   rrx.try_recv();
                               }
                               flag_count = 4;             // 原来这里是4，下一采样点如果仍是增长，继续触发丢包。这里用0，宽松一点，减少丢包频率。
                           }
                           flag_count += 1;
                       } else {
                           flag_count = 0;
                       } }
                    }
            }) => { },
            _ = tokio::spawn(async move {stop_for_send_thread.recv().await}) =>{
                println!("send closed");
            }
        }
    });

    Ok(tx)
}
