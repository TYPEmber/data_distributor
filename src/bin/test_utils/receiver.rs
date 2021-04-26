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
    #[structopt(long)]
    time_limit_enable: bool,
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

async fn recv_tcp_u32(stream: &TcpStream, buffer_length:usize) -> Result<u32, Box<dyn Error>> {
    //let mut buf = [0; 5000];
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
    let res = u32::from_ne_bytes(buf[0..4].try_into().unwrap());
    Ok(res)
}

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
    let mut duration_time=0.0;
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
                        duration_time=watch.elapsed().unwrap().as_secs_f64();
                    }
                }

                    } => {  // 正常接收数据包后记录结束时间
                            //let duration_time = watch.elapsed().unwrap().as_secs_f64();
                            println!("mission complete! the number out of order is {}，{}，{}",num_out_of_order,recv_pkg_count,duration_time);
                            (recv_pkg_count,num_out_of_order,duration_time)
                         },

           _= async{
               // 有丢包情况下，接收一个tcp包来结束任务
               recv_tcp_u32(&stream_stop,buffer_length).await;
           }   =>{
               //let duration_time = watch.elapsed().unwrap().as_secs_f64()-10.0;
               println!("time out!");
               (recv_pkg_count,num_out_of_order,duration_time)
                           },
        };
        t_result
    }).await.unwrap();
    result
}


async fn test_recv_packages_time_limt(
    time_limit: f64,
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
    let mut duration_time=0.0;
      let t_result=  tokio::select! {
            _ = async  {
                loop{
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
                        duration_time=watch.elapsed().unwrap().as_secs_f64();
                    }
                }

                    } => {  // 正常接收数据包后记录结束时间
                            //let duration_time = watch.elapsed().unwrap().as_secs_f64();
                            println!("mission complete! the number out of order is {}，{}，{}",num_out_of_order,recv_pkg_count,duration_time);
                            (recv_pkg_count,num_out_of_order,duration_time)
                         },

           _= async{
               // sender发送完数据包后，接收一个tcp包来结束任务
               recv_tcp_u32(&stream_stop,buffer_length).await;
           }   =>{
              // let duration_time = watch.elapsed().unwrap().as_secs_f64()-10.0;
               println!("time out! the duration time is{}",duration_time);
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
    if cmd.time_limit_enable{
        recv_pkg_time_limit(
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
    }else{
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
}
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
    let num_package = recv_tcp_u32(&stream_parameter,buffer_length).await.unwrap();

    // 接收发送速率
    let speed_rate = recv_tcp_f64(&stream_parameter,buffer_length).await.unwrap();

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
    let send_time_1 = recv_tcp_f64(&stream_parameter,buffer_length).await.unwrap();
    println!("the time is {:?}", send_time_1);

    // 开始第二轮接收数据包
    let result_2nd = test_recv_packages(num_package, buffer_length, socket, stream_stop_2nd).await;

    // 接收第二轮sender传输数据包的总时间
    let send_time_2 = recv_tcp_f64(&stream_parameter,buffer_length).await.unwrap();

    let calculate_num_1=(result.0/10000)*10000;
    let calculate_duration_1=(calculate_num_1/num_package) as f64 * send_time_1;
    
    let calculate_num_2=(result_2nd.0/10000)*10000;
    let calculate_duration_2=(calculate_num_2/num_package) as f64 * send_time_2;

    // 最后测试结果比较，打印
    println!("the calculate_duration_1 is {} and the result_time_1 is {}",calculate_duration_1,result.2);
    println!("the calculate_duration_2 is {} and the result_time_2 is {}",calculate_duration_2,result_2nd.2);
    println!("the speed rate is {}", speed_rate);

    println!(
        "1st round test: the sending packages are {}, and the receiving packages are {:?}",
        num_package, result.0
    );
    println!(
        "1st round test: the num out of order is {:?} and delay is {:?} ms",
        result.1,
        (result.2 - calculate_duration_1) / result.0 as f64 * 1000.0
    );
    println!(
        "2nd round test: the sending packages are {}, and the receiving packages are {:?}",
        num_package, result_2nd.0
    );
    println!(
        "2nd round test: the num out of order is {:?} and delay is {:?} ms",
        result_2nd.1,
        (result_2nd.2 - calculate_duration_2) / result_2nd.0 as f64 * 1000.0
    );
}
    

async fn recv_pkg_time_limit(
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

    // 接收发包总时间
    let time_duration = recv_tcp_f64(&stream_parameter,buffer_length).await.unwrap();

    // 接收发送速率
    let speed_rate = recv_tcp_f64(&stream_parameter,buffer_length).await.unwrap();

    // 开始第一轮接收，sender直接发送UDP包到receiver
    let result =
        test_recv_packages_time_limt(time_duration, buffer_length, socket_parameter, stream_stop).await;

    let mut buffer = vec![0u8; buffer_length];
    // 通知发送端第一轮完成
    buffer[0..4].copy_from_slice(&0u32.to_ne_bytes());
    match send_tcp(&stream_parameter, &buffer).await {
        Ok(_) => println!("1st round test is done."),
        Err(e) => println!("error {:?}", e),
    }

    // 接收第一轮sender发包总数量
    let send_pkg_num_1 = recv_tcp_u32(&stream_parameter,buffer_length).await.unwrap();
    println!("the send pkg num is {:?}", send_pkg_num_1);

    let send_pkg_duration_1=recv_tcp_f64(&stream_parameter,buffer_length).await.unwrap();
    println!("the send pkg time is {:?}", send_pkg_duration_1);

    // 开始第二轮接收数据包
    let result_2nd = test_recv_packages_time_limt(time_duration, buffer_length, socket, stream_stop_2nd).await;

    // 接收第二轮sender传输数据包的总时间
    let send_pkg_num_2 = recv_tcp_u32(&stream_parameter,buffer_length).await.unwrap();
    let send_pkg_duration_2=recv_tcp_f64(&stream_parameter,buffer_length).await.unwrap();
    println!("the send pkg time is {:?}", send_pkg_duration_2);
    
    let calculate_num_1=(result.0/10000)*10000; 
    let calculate_duration_1=(calculate_num_1/send_pkg_num_1) as f64 * send_pkg_duration_1;
    
    let calculate_num_2=(result_2nd.0/10000)*10000;
    let calculate_duration_2=(calculate_num_2/send_pkg_num_2) as f64 * send_pkg_duration_2;
    // 最后测试结果比较，打印
    println!("the calculate_duration_1 is {} and the result_time_1 is {}",calculate_duration_1,result.2);
    println!("the calculate_duration_2 is {} and the result_time_2 is {}",calculate_duration_2,result_2nd.2);
    println!("the speed rate is {} and the setting time_limit_duration is {}", speed_rate,time_duration);
    println!(
        "1st round test: the sending packages are {}, receiving packages are {} and the losing packages rate is {:?}",
        send_pkg_num_1,result.0, ((send_pkg_num_1 as f64-result.0 as f64) /send_pkg_num_1 as f64) 
    );
    println!(
        "1st round test: the num out of order is {:?} and delay is {:?} ms",
        result.1,
        (result.2 - calculate_duration_1) / result.0 as f64 * 1000.0
    );
    println!(
        "2nd round test: the sending packages are {},receiving packages are {} and the losing packages rate is {:?}",
        send_pkg_num_2,result_2nd.0, ((send_pkg_num_2 as f64-result_2nd.0 as f64) /send_pkg_num_2 as f64) 
    );
    println!(
        "2nd round test: the num out of order is {:?} and delay is {:?} ms",
        result_2nd.1,
        (result_2nd.2 - calculate_duration_2) / result_2nd.0 as f64 * 1000.0
    );
}