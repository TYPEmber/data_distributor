
use socket2::{Domain, SockAddr, Socket, Type};

use log::{debug, error, info, trace, warn};
use std::sync::{Arc, Mutex};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {}
}
///日志管理模块
/// 
pub mod logger;
///配置文件模块
pub mod params;
///Web服务器端功能模块
pub mod server;

/// 根据配置参数进行初始化
pub fn initial(
    distributors: Vec<(usize, std::net::SocketAddr, Vec<std::net::SocketAddr>)>,
    send_buff_size: usize,
    stop_trigger: tokio::sync::broadcast::Sender<()>,
) -> Result<
    (
        Vec<Distributor>,
        Arc<HashMap<std::net::SocketAddr, tokio::sync::mpsc::Sender<SendRequest>>>,
    ),
    std::io::Error,
> {
    let mut dis_vec = vec![];

    for (recv_buff_size, local_addr, remote_addrs) in distributors.into_iter() {
        dis_vec.push(Distributor::new(
            recv_buff_size,
            local_addr,
            remote_addrs,
            stop_trigger.clone(),
        )?);
    }

    let dis_vec = dis_vec;

    let map = generate_sender_map(&dis_vec, stop_trigger.clone(), send_buff_size)?;

    Ok((dis_vec, map))
}
/// 启动软件运行，开始进行数据分发。
pub async fn run(
    dis_vec: Vec<Distributor>,
    map: Arc<HashMap<std::net::SocketAddr, tokio::sync::mpsc::Sender<SendRequest>>>,
) {
    for dis_obj in dis_vec.into_iter() {
        dis_obj.run(map.clone()).await;
    }
}
/// 根据给定的IP地址，发送，接收缓存，绑定生成一个UDP Socket。
pub fn generate_socket(
    bind_addr: std::net::SocketAddr,
    recv_buff_size: usize,
    send_buff_size: usize,
) -> Result<tokio::net::UdpSocket, std::io::Error> {
    let sender = Socket::new(Domain::IPV4, Type::DGRAM, None)?;
    sender.bind(&SockAddr::from(bind_addr))?;
    sender.set_nonblocking(true).unwrap();
    sender.set_recv_buffer_size(recv_buff_size)?;
    sender.set_send_buffer_size(send_buff_size)?;
    let sender = sender.into();
    tokio::net::UdpSocket::from_std(sender)
}
use std::collections::{HashMap, HashSet};
/// 负责数据收发的结构体
pub struct Distributor {
    /// 监听前方站点的IP地址端口号
    pub receiver: tokio::net::UdpSocket,
    /// 用于实时统计数据接收的BPS
    pub recv_speed: Arc<AtomicUsize>,
    /// 记录每一秒收到的数据量
    pub recv_speed_acc: Arc<AtomicUsize>,
    ///用于实时统计数据接收的PPS
    pub recv_pkg_speed: Arc<AtomicUsize>,
    /// 记录每一秒收到的包数量
    pub recv_pkg_speed_acc: Arc<AtomicUsize>,
    /// MPSC channel的一个发送端，用于发送STOP命令，结束数据收发进程
    pub stop_broadcast_sender: tokio::sync::broadcast::Sender<()>,
    /// 待分发的远程IP地址组
    pub remote: Vec<Arc<RemoteInfo>>,
}

impl Distributor {
    /// 根据配置参数，生成一个具体的Distributor结构体实例
    pub fn new(
        recv_buff_size: usize,
        listen_addr: std::net::SocketAddr,
        remote_addrs: Vec<std::net::SocketAddr>,
        stop_broadcast_sender: tokio::sync::broadcast::Sender<()>,
    ) -> Result<Self, std::io::Error> {
        Ok(Self {
            receiver: generate_socket(listen_addr, recv_buff_size, 1024)?,
            recv_speed: Arc::new(AtomicUsize::new(0)),
            recv_speed_acc: Arc::new(AtomicUsize::new(0)),
            recv_pkg_speed: Arc::new(AtomicUsize::new(0)),
            recv_pkg_speed_acc: Arc::new(AtomicUsize::new(0)),
            remote: remote_addrs
                .iter()
                .map(|addr| {
                    Arc::new(RemoteInfo {
                        addr: *addr,
                        speed: AtomicUsize::new(0),
                        speed_acc: AtomicUsize::new(0),
                        pkg_speed: AtomicUsize::new(0),
                        pkg_speed_acc: AtomicUsize::new(0),
                    })
                })
                .collect(),
            stop_broadcast_sender: stop_broadcast_sender,
        })
    }
    /// 新建一个Distributor的数据分发线程，该线程包括三个异步任务，
    /// 当其中任何一个异步任务率先完成时，则其他两个异步任务也同时结束
    /// # 数据接收
    /// 通过绑定的IP地址端口号，接收前方站点发回的数据包。并将其打包转换成
    /// SendRequest形式，根据收到包的目的IP地址，通过MPSC通道，发送给对应
    /// 的网卡发送线程。
    /// # 数据统计
    /// 实时统计当前接收数据包的BPS和PPS，每间隔一秒记录一次，并将结果保存
    /// 到logger的实例中。
    /// # 接收STOP指令
    /// 一个专门用于结束数据收发线程的MPSC通道。在Web客户端执行STOP操作后，
    /// Web服务器端接收该HTTP请求后，会通过该通道，在通道发送端发送一个STOP消息，
    /// 对应接收端收到消息后，结束任务。由于三个异步任务都在select内执行，则该异步
    /// 任务结束后，其余两个数据接收，数据统计任务也会同时退出。
    pub async fn run(
        mut self,
        sender_map: Arc<HashMap<std::net::SocketAddr, tokio::sync::mpsc::Sender<SendRequest>>>,
    ) {
        let mut stop_broadcast_recv = self.stop_broadcast_sender.subscribe();

        let recv_speed = self.recv_speed.clone();
        let recv_speed_acc = self.recv_speed_acc.clone();
        let recv_pkg_speed = self.recv_pkg_speed.clone();
        let recv_pkg_speed_acc = self.recv_pkg_speed_acc.clone();

        let local_addr = self.receiver.local_addr().unwrap();

        let remotes = self.remote.clone();
        let mut buf = Box::new([0u8; 65536]);

        // receive thread
        tokio::spawn(async move {
            tokio::select! {
                _ = async  {
                        loop {
                        if let Ok((len, _)) = self.receiver.recv_from(&mut buf[..]).await{
                            self.recv_speed_acc.fetch_add(len, Ordering::Relaxed);
                            self.recv_pkg_speed_acc.fetch_add(1, Ordering::Relaxed);
                            let data = Arc::new(buf[..len].to_vec());
                            for remote in &self.remote {
                                if let Err(e) = sender_map
                                    .get(&remote.addr)
                                    .unwrap()
                                    .try_send(SendRequest::new(remote.clone(), data.clone()))
                                    {
                                        //error!("[{}]", e);
                                    }
                            }
                        }

                }} => { },

                 // caculate speed
            _ = async  {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(1_000));
                loop{
                    interval.tick().await;
                    // data speed
                    recv_speed.store(recv_speed_acc.load(Ordering::Relaxed), Ordering::Relaxed);
                    recv_speed_acc.store(0, Ordering::Relaxed);
                    // pkg speed
                    recv_pkg_speed.store(recv_pkg_speed_acc.load(Ordering::Relaxed), Ordering::Relaxed);
                    recv_pkg_speed_acc.store(0, Ordering::Relaxed);
                    info!("SPEED IN {} {} {}", local_addr, 8 * recv_speed.load(Ordering::Relaxed), recv_pkg_speed.load(Ordering::SeqCst));
                    for remote in &remotes {
                        // data speed
                        remote
                            .speed
                            .store(remote.speed_acc.load(Ordering::Relaxed), Ordering::Relaxed);
                        remote.speed_acc.store(0, Ordering::Relaxed);
                        // pkg speed
                        remote
                        .pkg_speed
                        .store(remote.pkg_speed_acc.load(Ordering::Relaxed), Ordering::Relaxed);
                    remote.pkg_speed_acc.store(0, Ordering::Relaxed);
                        info!(
                            "SPEED OUT {} {} {}",
                            remote.addr,
                            8 * remote.speed.load(Ordering::Relaxed),
                            remote.pkg_speed.load(Ordering::Relaxed)
                        );
                    }
                }
            } => {},
            _ = stop_broadcast_recv.recv() => {
                let mut msg = "".to_owned();
                for remote in &remotes {
                    msg.push_str(" OUT_");
                    msg.push_str(&remote.addr.to_string()[..]);
                }
                info!("CLOSED IN_{}{}", local_addr, msg)
            },
            };
        });
    }
}
/// 生成一个sender_map，为一个包含Remote_addr和对应网卡发送线程的MPSC发送端的HashMap。
/// 
///该函数会为当前软件运行时，配置参数里的每一个Remote_addr, 都绑定到其中一个
/// 网卡发送线程。在绑定以后，所有发往该Remote_addr的数据包，都会通过该网卡发
/// 送线程进行发送。
pub fn generate_sender_map(
    distributors: &Vec<Distributor>,
    stop_broadcast_sender: tokio::sync::broadcast::Sender<()>,
    send_buff_size: usize,
) -> Result<
    Arc<HashMap<std::net::SocketAddr, tokio::sync::mpsc::Sender<SendRequest>>>,
    std::io::Error,
> {
    let mut map = HashMap::new();
    let mut local_ips = HashMap::new();
    // It will choose the same sender port for every net card
    let socket = std::net::UdpSocket::bind("0.0.0.0:0")?;
    for item in distributors {
        for remote in &item.remote {
            socket.connect(remote.addr)?;
            let local = socket.local_addr()?;

            if !local_ips.contains_key(&local) {
                local_ips.insert(
                    local,
                    generate_sender_thread(stop_broadcast_sender.clone(), send_buff_size)?,
                );
            }
            map.insert(remote.addr, local_ips.get(&local).unwrap().clone());
        }
    }

    //println!("{:?}", local_ips);

    Ok(Arc::new(map))
}

use std::sync::atomic::{AtomicUsize, Ordering};
/// 该结构体定义了SendRequest中Remote成员所需要的全部信息
pub struct RemoteInfo {
    // Remote_addr
    pub addr: std::net::SocketAddr,
    /// 用于统计发往对应Remote_addr的BPS
    pub speed: AtomicUsize,
    /// 用于记录每秒发送的bit量
    pub speed_acc: AtomicUsize,
    /// 用于统计发往对应Remote_addr的PPS
    pub pkg_speed: AtomicUsize,
    /// 用于记录每秒发送的包数量
    pub pkg_speed_acc: AtomicUsize,
}
/// 该结构体包含网卡发送线程发送到remote_addr所需要的全部信息。
/// 
/// data是一个u8数组，为收到的数据包数据。
/// remote是一个RemoteInfo数据类型。
pub struct SendRequest {
    /// remote是一个RemoteInfo数据类型
    pub remote: Arc<RemoteInfo>,
    ///data是一个u8数组，为收到的数据包数据
    pub data: Arc<Vec<u8>>,
}
impl SendRequest {
    /// 新生成一个SendRequest实例
    pub fn new(remote: Arc<RemoteInfo>, data: Arc<Vec<u8>>) -> Self {
        Self { remote, data }
    }
}

/// 该函数生成一个网卡发送线程。
/// 
/// MPSC通道最大队列数量被设置为2048，当缓存数量溢出时，则会产生丢包行为，
/// 来减少因为缓存过大，而导致的UDP包分发延时过大。
pub fn generate_sender_thread(
    stop_broadcast_sender: tokio::sync::broadcast::Sender<()>,
    send_buff_size:usize,
) -> Result<tokio::sync::mpsc::Sender<SendRequest>, std::io::Error> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<SendRequest>(2048);
    let socket = generate_socket("0.0.0.0:0".parse().unwrap(), 1024, send_buff_size)?;

    let mut stop_for_send_thread = stop_broadcast_sender.subscribe();
    let mut stop_for_send_thread_1 = stop_broadcast_sender.subscribe();
    let mut send_size = Arc::new(AtomicUsize::new(0));
    let mut send_size1 = send_size.clone();
    tokio::spawn(async move {
        tokio::select! {
           _ = async {
                let mut interval =  tokio::time::interval(tokio::time::Duration::from_millis(1));
                loop {
                    //tokio::task::yield_now().await;
                    if let Some(req) = rx.recv().await {
                        match socket
                            .send_to(&req.data[..], req.remote.addr)
                            .await{
                                Ok(len)=>{
                                    req.remote.speed_acc.fetch_add(len, Ordering::Relaxed);
                                    req.remote.pkg_speed_acc.fetch_add(1, Ordering::Relaxed);
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
            }=>{},
            _ = tokio::spawn(async move {stop_for_send_thread.recv().await}) =>{
                println!("send closed");
            }
        }
    });

    Ok(tx)
}
