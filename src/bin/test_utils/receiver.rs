use data_distributor::*;
use log::{debug, error, info, trace, warn};
use std::convert::TryInto;
use std::error::Error;
use std::str;
use std::{convert::Infallible, net::SocketAddr};
use structopt::StructOpt;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};
#[derive(StructOpt, Debug)]
pub struct Opt {
    #[structopt(short, long, default_value = "4194304")]
    recv_buffer: usize,
    #[structopt(long, default_value = "4096")]
    send_buffer: usize,
    #[structopt(short, long, default_value = "1000000000.0")]
    speed_rate: f64,
    #[structopt(short, long, default_value = "1000000")]
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
    #[structopt(short, long, default_value = "127.0.0.1:19208")]
    distributor_addr: String,
}

async fn send_tcp(stream: &TcpStream, buf: &[u8]) -> Result<(), Box<dyn Error>> {
    loop {
        stream.writable().await?;
        match stream.try_write(buf) {
            Ok(n) => {
                break;
            }

            Err(e) => {
                println!("{}", e);
                return Err(e.into());
            }
        }
    }
    Ok(())
}

async fn recv_tcp_u32(stream: &TcpStream) -> Result<u32, Box<dyn Error>> {
    let mut buf = [0; 1350];
    loop {
        stream.readable().await?;
        match stream.try_read(&mut buf) {
            Ok(n) => {
                break;
            }
            Err(e) => {}
        }
    }
    let res = u32::from_ne_bytes(buf[0..4].try_into().unwrap());
    Ok(res)
}

async fn recv_tcp_f64(stream: &TcpStream) -> Result<f64, Box<dyn Error>> {
    let mut buf = [0; 1350];
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

async fn test_recv_packages(
    num_package: u32,
    buffer_length: usize,
    socket: tokio::net::UdpSocket,
    stream_stop: tokio::net::TcpStream,
) -> (u32, u32, f64) {
    let mut recv_id: u32 = 0;
    let mut recv_pkg_count = 0u32;
    let mut num_out_of_order = 0u32;
    let mut current_index = 0u32;
    let mut buffer = vec![0u8; buffer_length];

    // 开始记录接收数据包时间
    let mut watch = std::time::SystemTime::now();
    let  result=   tokio::spawn(async move {
      let t_result=  tokio::select! {
            _ = async  {
                while recv_pkg_count < num_package {
                socket.recv(&mut buffer).await;
                    recv_id=u32::from_ne_bytes(buffer[0..4].try_into().unwrap());
                    // 检查收到的数据包是否乱序
                  if recv_id<current_index{
                       num_out_of_order+=1;
                   }
                   current_index=recv_id;
                    recv_pkg_count += 1;
                    if recv_pkg_count % 10000 == 0 {
                        println!("has recv: {}", recv_pkg_count);
                    }
                }

                    } => {  // 正常接收数据包后记录结束时间
                            let duration_time = watch.elapsed().unwrap().as_secs_f64();
                            println!("mission complete! the number out of order is {}，{}，{}",num_out_of_order,recv_pkg_count,duration_time);
                            (recv_pkg_count,num_out_of_order,duration_time)
                         },

           _= async{
               // 有丢包情况下，接收一个tcp包来结束任务
               recv_tcp_u32(&stream_stop).await;
           }   =>{
               let duration_time = watch.elapsed().unwrap().as_secs_f64()-10.0;
               println!("time out!");
               (recv_pkg_count,num_out_of_order,duration_time)
                           },
        };
        t_result
    }).await.unwrap();
    result
}

#[tokio::main]
async fn main() {
    // 获得命令行参数
    let cmd = Opt::from_args();
    let addr = cmd.direct_addr.parse::<std::net::SocketAddr>().unwrap();
    let addr1 = cmd
        .distributor_addr
        .parse::<std::net::SocketAddr>()
        .unwrap();
    let addr2 = cmd.tcp_addr.parse::<std::net::SocketAddr>().unwrap();
    let addr3 = cmd.tcp_addr_stop.parse::<std::net::SocketAddr>().unwrap();
    let addr4 = cmd
        .tcp_addr_stop_2nd
        .parse::<std::net::SocketAddr>()
        .unwrap();
    // 进行数据包接收
    recv_pkg(
        addr,
        addr1,
        addr2,
        addr3,
        addr4,
        cmd.buffer_length,
        cmd.recv_buffer,
        cmd.send_buffer,
    )
    .await;

    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(1_000)).await;
    }
}

async fn recv_pkg(
    direct_ip: std::net::SocketAddr,
    distributor_ip: std::net::SocketAddr,
    tcp_ip: std::net::SocketAddr,
    tcp_ip_stop: std::net::SocketAddr,
    tcp_ip_stop_2nd: std::net::SocketAddr,
    buffer_length: usize,
    recv_buffer: usize,
    send_buffer: usize,
) {
    let mut socket = crate::generate_socket(distributor_ip, recv_buffer, send_buffer).unwrap();
    let mut socket_parameter = crate::generate_socket(direct_ip, recv_buffer, send_buffer).unwrap();

    let listener = TcpListener::bind(tcp_ip).await.unwrap();
    let (stream_parameter, _) = listener.accept().await.unwrap();

    let listener_stop = TcpListener::bind(tcp_ip_stop).await.unwrap();
    let (stream_stop, _) = listener_stop.accept().await.unwrap();

    let listener_stop_2nd = TcpListener::bind(tcp_ip_stop_2nd).await.unwrap();
    let (stream_stop_2nd, _) = listener_stop_2nd.accept().await.unwrap();

    let mut buffer = vec![0u8; buffer_length];

    // 接收发包总数
    let num_package = recv_tcp_u32(&stream_parameter).await.unwrap();

    // 接收发送速率
    let speed_rate = recv_tcp_f64(&stream_parameter).await.unwrap();

    // 开始第一轮接收，sender直接发送UDP包到receiver
    let result =
        test_recv_packages(num_package, buffer_length, socket_parameter, stream_stop).await;

    let mut buffer = vec![0u8; buffer_length];
    // 通知发送端第一轮完成
    buffer[0..4].copy_from_slice(&0u32.to_ne_bytes());
    match send_tcp(&stream_parameter, &buffer).await {
        Ok(_) => println!("1st round test is done."),
        Err(e) => println!("error {:?}", e),
    }

    // 接收第一轮sender发包的总时间
    let send_time_1 = recv_tcp_f64(&stream_parameter).await.unwrap();
    println!("the time is {:?}", send_time_1);

    // 开始第二轮接收数据包
    let result_2nd = test_recv_packages(num_package, buffer_length, socket, stream_stop_2nd).await;

    // 接收第二轮sender传输数据包的总时间
    let send_time_2 = recv_tcp_f64(&stream_parameter).await.unwrap();

    // 最后测试结果比较，打印
    println!("the speed rate is {}", speed_rate);
    println!(
        "1st round test: the sending packages are {}, and the receiving packages are {:?}",
        num_package, result.0
    );
    println!(
        "1st round test: the num out of order is {:?} and delay is {:?} ms",
        result.1,
        (result.2 - send_time_1) / (((num_package + result.0) as f64) * 0.5) * 1000.0
    );
    println!(
        "2nd round test: the sending packages are {}, and the receiving packages are {:?}",
        num_package, result_2nd.0
    );
    println!(
        "2nd round test: the num out of order is {:?} and delay is {:?} ms",
        result_2nd.1,
        (result_2nd.2 - send_time_2) / (((num_package + result_2nd.0) as f64) * 0.5) * 1000.0
    );
}
