use data_distributor::*;
<<<<<<< HEAD
=======
use log::{debug, error, info, trace, warn};
use socket2::{Domain, SockAddr, Socket, Type};
use std::convert::TryInto;
use std::error::Error;
use std::{convert::Infallible, net::SocketAddr};
use structopt::StructOpt;
use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};

#[derive(StructOpt, Debug)]
pub struct Opt {
    #[structopt(short, long, default_value = "4096")]
    recv_buffer: usize,
    #[structopt(long, default_value = "4194304")]
    send_buffer: usize,
    #[structopt(short, long, default_value = "1000000000.0")]
    speed_rate: f64,
    #[structopt(short, long, default_value = "2000000")]
    package_count: u32,
    #[structopt(short, long, default_value = "1350")]
    buffer_length: usize,
    #[structopt(long, default_value = "127.0.0.1:19209")]
    direct_addr: String,
    #[structopt(short, long, default_value = "127.0.0.1:19211")]
    tcp_addr: String,
    #[structopt(short, long, default_value = "127.0.0.1:19212")]
    tcp_addr_stop: String,
    #[structopt(short, long, default_value = "127.0.0.1:19850")]
    tcp_addr_stop_2nd: String,
    #[structopt(short, long, default_value = "127.0.0.1:5503")]
    distributor_addr: String,
    #[structopt(short, long, default_value = "80000.0")]
    package_rate: f64,
    #[structopt(long)]
    pps_enable: bool,
    #[structopt(long)]
    time_limit_enable: bool,
    #[structopt(short, long, default_value = "30.0")]
    time_duration: f64,
}

async fn send_tcp(stream: &TcpStream, buf: &[u8]) -> Result<(), Box<dyn Error>> {
    loop {
        stream.writable().await?;
        match stream.try_write(buf) {
            Ok(n) => {
                break;
            }
>>>>>>> 3f9554bf5425af72a446f14a71cd5081470542e1

#[tokio::main]
async fn main() {
    recv_pkg("0.0.0.0:19208".parse().unwrap(), 100_000_0).await;
    send_pkg("127.0.0.1:5507".parse().unwrap(), 100_000_0, 9e8).await;

<<<<<<< HEAD
=======
async fn recv_tcp_u32(stream: &TcpStream, buffer_length:usize) -> Result<u32, Box<dyn Error>> {
    //let mut buf = [0; 5000];
    let mut buf = vec![0u8; buffer_length];
>>>>>>> 3f9554bf5425af72a446f14a71cd5081470542e1
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(1_000)).await;
    }
}

<<<<<<< HEAD
use socket2::{Domain, SockAddr, Socket, Type};

async fn send_pkg(addr: std::net::SocketAddr, pkg_count: usize, speed_rate: f64) {
    let any_addr: std::net::SocketAddr = "0.0.0.0:0".parse().unwrap();
    let mut socket = crate::generate_socket(any_addr, 4096, 4096 * 1024).unwrap();

=======
async fn recv_tcp_f64(stream: &TcpStream,buffer_length:usize) -> Result<f64, Box<dyn Error>> {
   // let mut buf = [0; 5000];
   let mut buf = vec![0u8; buffer_length];
    loop {
        stream.readable().await?;
        match stream.try_read(&mut buf) {
            Ok(n) => {
                break;
            }
            Err(e) => {}
        }
    }
    let res = f64::from_ne_bytes(buf[0..8].try_into().unwrap());
    Ok(res)
}

async fn test_send_packages(
    pkg_num: u32,
    speed_rate: f64,
    buffer_length: usize,
    socket: &tokio::net::UdpSocket,
    direct_ip: std::net::SocketAddr,
    package_rate:f64,
    pps_enable: bool,
) -> f64 {
>>>>>>> 3f9554bf5425af72a446f14a71cd5081470542e1
    let mut time = std::time::SystemTime::now();
    let mut last_print_time = 0usize;
    let mut send_bits_count_last = 0usize;
    let mut send_bits_count = 0usize;
    let mut send_pkg_count = 0usize;
    let mut speed_now = 0.0;

    let data = [0u8; 1350];

    let mut watch = std::time::SystemTime::now();

    while send_pkg_count < pkg_count {
        let mut time_f64 = time.elapsed().unwrap().as_secs_f64();
        if send_pkg_count % 1 == 0 {
<<<<<<< HEAD
            speed_now = send_bits_count as f64 / time_f64;
            //println!("{}", time.elapsed().unwrap().as_secs_f64());
            while (speed_now > speed_rate) {
=======
          // pps_enable为true时，按pps速率来进行发送，否则按bps速率发送
         if pps_enable {
             // 实时统计当前的包发送速率
            speed_now = send_pkg_count as f64 / time_f64;
            // 如果发送速率大于预设包发送速率pps，控制速率
            while speed_now > package_rate {
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;

                speed_now = send_pkg_count as f64 / time.elapsed().unwrap().as_secs_f64()
            } 

         } else{
            // 实时统计当前的数据量发送速率
            speed_now = send_bits_count as f64 / time_f64;
            // 如果发送速率大于预设bps发送速率，控制速率
            while speed_now > speed_rate {
>>>>>>> 3f9554bf5425af72a446f14a71cd5081470542e1
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                speed_now = send_bits_count as f64 / time.elapsed().unwrap().as_secs_f64()
            } }
        }

        let time_usize = time_f64 as usize;

        if time_usize - last_print_time >= 1 {
            println!("{:?}", speed_now);
            last_print_time = time_usize;
        }

        if let Ok(len) = socket.send_to(&data, addr).await {
            // if len != 1350 {
            //     panic!();
            // }
            send_bits_count += len * 8;
        }
        send_pkg_count += 1;
    }

    let du = watch.elapsed().unwrap().as_secs_f64();

    println!(
        "send_pkg_count: {} speed: {} duration: {}",
        send_pkg_count, speed_now, du
    );
}

<<<<<<< HEAD
async fn recv_pkg(addr: std::net::SocketAddr, pkg_count: usize) {
    let mut socket = crate::generate_socket(addr, 4096 * 4096 * 10, 4096).unwrap();

    let mut recv_pkg_count = 0usize;

    tokio::spawn(async move {
        let mut buffer = [0u8; 5000];
        while recv_pkg_count < pkg_count {
            socket.recv(&mut buffer).await;
            recv_pkg_count += 1;

            if recv_pkg_count % 10000 == 0 {
                println!("has recv: {}", recv_pkg_count);
            }
        }
        println!("mission complete");
    });
=======


async fn test_send_packages_time_limit(
    pkg_num: u32,
    speed_rate: f64,
    buffer_length: usize,
    socket: &tokio::net::UdpSocket,
    direct_ip: std::net::SocketAddr,
    package_rate:f64,
    pps_enable: bool,
    limit_time:f64,
) -> (u32,f64) {
    let mut time = std::time::SystemTime::now();
    let mut last_print_time = 0usize;
    let mut send_bits_count_last = 0usize;
    let mut send_bits_count = 0usize;
    let mut send_pkg_count = 0u32;
    let mut speed_now: f64 = 0.0;
    let mut arr = vec![0u8; buffer_length];
    let mut data = &mut arr[..];
    let mut watch = std::time::SystemTime::now();
    let mut time_duration=watch.elapsed().unwrap().as_secs_f64();
    while time_duration <= limit_time {
        // 开始记录测试时间包发送时间
        let mut time_f64 = time.elapsed().unwrap().as_secs_f64();
        if send_pkg_count % 1 == 0 {
          // pps_enable为true时，按pps速率来进行发送，否则按bps速率发送
         if pps_enable {
             // 实时统计当前的包发送速率
            speed_now = send_pkg_count as f64 / time_f64;
            // 如果发送速率大于预设包发送速率pps，控制速率
            while speed_now > package_rate {
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;

                speed_now = send_pkg_count as f64 / time.elapsed().unwrap().as_secs_f64()
            } 

         } else{
            // 实时统计当前的数据量发送速率
            speed_now = send_bits_count as f64 / time_f64;
            // 如果发送速率大于预设bps发送速率，控制速率
            while speed_now > speed_rate {
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;

                speed_now = send_bits_count as f64 / time.elapsed().unwrap().as_secs_f64()
            } }
        }

        let time_usize = time_f64 as usize;

        if time_usize - last_print_time >= 1 {
            println!("{:?}", speed_now);
            last_print_time = time_usize;
        }
        // 每个发送的测试数据包，都会包含id信息，用于测试recerver端接收是否发生乱序
        let id_bytes = send_pkg_count.to_ne_bytes();
        data[0..4].copy_from_slice(&id_bytes[0..]);
        // 根据递增形式给每个包一个id
        if let Ok(len) = socket.send_to(&data, direct_ip).await {
            // 直接发送给目标地址
            if len != buffer_length {
                panic!();
            }
            send_bits_count += len * 8;
        }

        send_pkg_count += 1;
        time_duration=watch.elapsed().unwrap().as_secs_f64();
    }

    // 记录发包的总时间
  //  let duration_time = watch.elapsed().unwrap().as_secs_f64();
    println!("the send time duration is {}",time_duration);
    (send_pkg_count,time_duration)
}














#[tokio::main]
async fn main() {
    // 获取命令行参数
    let cmd = Opt::from_args();

    let addr = cmd.direct_addr.parse::<std::net::SocketAddr>().unwrap();
    let addr1 = cmd.tcp_addr.parse::<std::net::SocketAddr>().unwrap();
    let addr2 = cmd
        .distributor_addr
        .parse::<std::net::SocketAddr>()
        .unwrap();
    let addr3 = cmd.tcp_addr_stop.parse::<std::net::SocketAddr>().unwrap();
    let addr4 = cmd
        .tcp_addr_stop_2nd
        .parse::<std::net::SocketAddr>()
        .unwrap();
    if cmd.time_limit_enable{
        send_pkg_time_limit(
            cmd.package_count,
            cmd.speed_rate,
            cmd.send_buffer,
            cmd.recv_buffer,
            addr,
            addr1,
            addr2,
            addr3,
            addr4,
            cmd.buffer_length,
            cmd.package_rate,
            cmd.pps_enable,
            cmd.time_duration,

        ).await;
    }else{
    send_pkg(
        cmd.package_count,
        cmd.speed_rate,
        cmd.send_buffer,
        cmd.recv_buffer,
        addr,
        addr1,
        addr2,
        addr3,
        addr4,
        cmd.buffer_length,
        cmd.package_rate,
        cmd.pps_enable,
    )
    .await;
}
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(1_000)).await;
    }
}

async fn send_pkg(
    pkg_count: u32,
    speed_rate: f64,
    send_buffer: usize,
    recv_buffer: usize,
    direct_ip: std::net::SocketAddr,
    tcp_ip: std::net::SocketAddr,
    distributor_ip: std::net::SocketAddr,
    tcp_ip_stop: std::net::SocketAddr,
    tcp_ip_stop_2nd: std::net::SocketAddr,
    buffer_length: usize,
    package_speed_rate:f64,
    pps_enable: bool,
) {
    let any_addr: std::net::SocketAddr = "0.0.0.0:0".parse().unwrap();
    let mut socket = crate::generate_socket(any_addr, recv_buffer, send_buffer).unwrap();
    let stream = TcpStream::connect(tcp_ip).await.unwrap();
    let stream_stop = TcpStream::connect(tcp_ip_stop).await.unwrap();
    let stream_stop_2nd = TcpStream::connect(tcp_ip_stop_2nd).await.unwrap();

    let mut arr = vec![0u8; buffer_length];
    let mut data = &mut arr[..];

    // 发包总数
    data[0..4].copy_from_slice(&pkg_count.to_ne_bytes());
    send_tcp(&stream, data).await.unwrap();

    // 发包速率
    data[0..8].copy_from_slice(&speed_rate.to_ne_bytes());
    send_tcp(&stream, data).await.unwrap();

    //开始第一轮测试
    let mut duration_time =
        test_send_packages(pkg_count, speed_rate, buffer_length, &socket, direct_ip,package_speed_rate,pps_enable).await;
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    // 如果有丢包现象，发送一个0u32的测试包结束第一轮接收
    data[0..4].copy_from_slice(&0u32.to_ne_bytes());
    send_tcp(&stream_stop, data).await.unwrap();

    // 收到receiver端反馈，第一轮测试结束
    match recv_tcp_u32(&stream,buffer_length).await {
        Ok(_) => println!("Now,starting 2nd round test."),
        Err(e) => println!("error {:?}", e),
    }

    // 发送包发送时间
    data[0..8].copy_from_slice(&duration_time.to_ne_bytes());
    send_tcp(&stream, data).await.unwrap();

    // 开始第二轮测试，从distributor转发到目标地址

    // 第二次发送数据包结束时间
    duration_time = test_send_packages(
        pkg_count,
        speed_rate,
        buffer_length,
        &socket,
        distributor_ip,
        package_speed_rate,
        pps_enable,
    )
    .await;
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    // 发送一个结束信号给receiver
    data[0..4].copy_from_slice(&0u32.to_ne_bytes());
    send_tcp(&stream_stop_2nd, data);

    // 发送第二轮sender包发送时间
    data[0..8].copy_from_slice(&duration_time.to_ne_bytes());
    send_tcp(&stream, data).await.unwrap();
    println!("the time is {}", duration_time);
>>>>>>> 3f9554bf5425af72a446f14a71cd5081470542e1
}


async fn send_pkg_time_limit(
    pkg_count: u32,
    speed_rate: f64,
    send_buffer: usize,
    recv_buffer: usize,
    direct_ip: std::net::SocketAddr,
    tcp_ip: std::net::SocketAddr,
    distributor_ip: std::net::SocketAddr,
    tcp_ip_stop: std::net::SocketAddr,
    tcp_ip_stop_2nd: std::net::SocketAddr,
    buffer_length: usize,
    package_speed_rate:f64,
    pps_enable: bool,
    time_limit:f64,
) {
    let any_addr: std::net::SocketAddr = "0.0.0.0:0".parse().unwrap();
    let mut socket = crate::generate_socket(any_addr, recv_buffer, send_buffer).unwrap();
    let stream = TcpStream::connect(tcp_ip).await.unwrap();
    let stream_stop = TcpStream::connect(tcp_ip_stop).await.unwrap();
    let stream_stop_2nd = TcpStream::connect(tcp_ip_stop_2nd).await.unwrap();

    let mut arr = vec![0u8; buffer_length];
    let mut data = &mut arr[..];

    // 发包预设总时间
    data[0..8].copy_from_slice(&time_limit.to_ne_bytes());
    send_tcp(&stream, data).await.unwrap();

    // 发包速率
    data[0..8].copy_from_slice(&speed_rate.to_ne_bytes());
    send_tcp(&stream, data).await.unwrap();

    //开始第一轮测试
    let mut res =
        test_send_packages_time_limit(pkg_count, speed_rate, buffer_length, &socket, direct_ip,package_speed_rate,pps_enable,time_limit).await;
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    // 如果有丢包现象，发送一个0u32的测试包结束第一轮接收
    data[0..4].copy_from_slice(&0u32.to_ne_bytes());
    send_tcp(&stream_stop, data).await.unwrap();

    // 收到receiver端反馈，第一轮测试结束
    match recv_tcp_u32(&stream,buffer_length).await {
        Ok(_) => println!("Now,starting 2nd round test."),
        Err(e) => println!("error {:?}", e),
    }

    // 发送规定时间内发送的包数量
    data[0..4].copy_from_slice(&res.0.to_ne_bytes());
    send_tcp(&stream, data).await.unwrap();

    // 发送时间发送包的时间
    data[0..8].copy_from_slice(&res.1.to_ne_bytes());
    send_tcp(&stream, data).await.unwrap();

    // 开始第二轮测试，从distributor转发到目标地址

    // 第二次发送数据包结束时间
    res = test_send_packages_time_limit(
        pkg_count,
        speed_rate,
        buffer_length,
        &socket,
        distributor_ip,
        package_speed_rate,
        pps_enable,
        time_limit,
    )
    .await;
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

    // 发送一个结束信号给receiver
    data[0..4].copy_from_slice(&0u32.to_ne_bytes());
    send_tcp(&stream_stop_2nd, data);

    // 发送第二轮sender包发送数量和时间
    data[0..4].copy_from_slice(&res.0.to_ne_bytes());
    send_tcp(&stream, data).await.unwrap();

    data[0..8].copy_from_slice(&res.1.to_ne_bytes());
    send_tcp(&stream, data).await.unwrap();
    println!("the number is {} and the send_time is {}", res.0,res.1);
}