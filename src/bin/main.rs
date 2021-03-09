use data_distributor::*;

#[macro_use]
extern crate structopt;

//use std::path::PathBuf;
use structopt::StructOpt;


#[derive(Debug, StructOpt)]
#[structopt(name = "distributor_command_line", about = "An implementation of command line for the distributor ")]
struct Opt {
    /// Activate debug mode
   // #[structopt(short = "d", long = "debug")]
    //debug: bool,
    /// Set receive_buffer_size
    #[structopt(short = "r", long = "receive", default_value = "1024")]
    receive_buff: usize,
    /// Input file
    //#[structopt(parse(from_os_str))]
    //input: PathBuf,
    /// Output file, stdout if not present
    //#[structopt(parse(from_os_str))]
    //output: Option<PathBuf>,
    //set the send_buffer_size
    #[structopt(short = "s", long = "send", default_value = "2048")]
    send_buff: usize,
    #[structopt(short = "i", long = "remote_address", default_value="127.0.0.1:19208 127.0.0.1:19210 192.168.200.3:19209 127.0.0.1:19211 127.0.0.1:19212 192.168.200.3:19201")]
    remote_address: String,
    #[structopt(short = "d", long = "distributor_address", default_value="127.0.0.1:5503 127.0.0.1:19210")]
    distributor_address: String,
    #[structopt(short = "m", long = "map", default_value="3 3")]
    map:String,
}




#[tokio::main]
async fn main() {
   
    let opt = Opt::from_args();
    let r_id: Vec<std::net::SocketAddr> = opt.remote_address.trim().split(' ').map(|x| x.parse().unwrap()).collect();
    let d_id: Vec<std::net::SocketAddr> = opt.distributor_address.trim().split(' ').map(|x| x.parse().unwrap()).collect();
    let num_split: Vec<usize> = opt.map.trim().split(' ').map(|x| x.parse().unwrap()).collect();
    
    let stop_sender = crate::initial(opt.send_buff, opt.receive_buff,r_id,d_id,num_split).await;         // set the socket size  
    recv_pkg("127.0.0.1:19208".parse().unwrap(), 100).await;
    send_pkg("127.0.0.1:5503".parse().unwrap(), 100, 5e8).await;    // set the count of package and expected speed rate  

    stop_sender.send(());

    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(1_000)).await; // parameter 2 
    }
}

async fn test_socket_send_limited() {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(5_000));
    let loop_times = 200_000_0;

    let one_socket_cost = tokio::spawn(async move {
        let socket = generate_socket("0.0.0.0:0".parse().unwrap(), 1024, 1024 * 1024*10);
        let addr: std::net::SocketAddr = "127.0.0.1:12345".parse().unwrap();
        let data = [0u8; 1350];

        let mut watch = std::time::SystemTime::now();
        let mut i = 0usize;
        while i < loop_times {
            socket.send_to(&data, addr).await;
            i += 1;
        }
        watch.elapsed().unwrap().as_secs_f64()
    })
    .await
    .unwrap();

    let multi_count = 5usize;

    let mut tasks = vec![];
    for k in 0..multi_count {
        let socket = generate_socket("0.0.0.0:0".parse().unwrap(), 1024, 1024 * 1024 * 10);
        let addr: std::net::SocketAddr = "127.0.0.1:12345".parse().unwrap();
        let data = [0u8; 1350];

        tasks.push(tokio::spawn(async move {
            let mut watch = std::time::SystemTime::now();
            let mut i = 0usize;
            while i < loop_times / multi_count {
                socket.send_to(&data, addr).await;
                i += 1;
            }
            watch.elapsed().unwrap().as_secs_f64()
        }));
    }
    let mut multi_socket_cost = 0f64;
    for t in tasks {
        multi_socket_cost += tokio::join!(t).0.unwrap();
    }

    println!(
        "{:?} {} bps",
        one_socket_cost,
        (loop_times * 1350 * 8) as f64 / one_socket_cost
    );
    println!(
        "{:?} {} bps",
        multi_socket_cost,
        (loop_times * 1350 * 8) as f64 / multi_socket_cost
    );
    panic!();
}

use socket2::{Domain, SockAddr, Socket, Type};

async fn send_pkg(addr: std::net::SocketAddr, pkg_count: usize, speed_rate: f64) {
    let any_addr: std::net::SocketAddr = "0.0.0.0:0".parse().unwrap();
    let mut socket = crate::generate_socket(any_addr, 1024, 1024* 10);    // parameter 6 

    let mut time = std::time::SystemTime::now();
    let mut last_print_time = 0usize;
    let mut send_bits_count_last = 0usize;
    let mut send_bits_count = 0usize;
    let mut send_pkg_count = 0usize;
    let mut speed_now = 0.0;

    let data = [0u8; 130];                // parameter 7  data size 

    let mut watch = std::time::SystemTime::now();

    while send_pkg_count < pkg_count {
        let mut time_f64 = time.elapsed().unwrap().as_secs_f64();
        if send_pkg_count % 1 == 0 {
            speed_now = send_bits_count as f64 / time_f64;
            //println!("{}", time.elapsed().unwrap().as_secs_f64());
            while (speed_now > speed_rate) {
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                speed_now = send_bits_count as f64 / time.elapsed().unwrap().as_secs_f64()    // control the speed of package sending 
            }
        }

        let time_usize = time_f64 as usize;

        if time_usize - last_print_time >= 1 {
            println!("{:?}", speed_now);
            last_print_time = time_usize;
        }

        if let Ok(len) = socket.send_to(&data, addr).await {
            if len != 130 {
                panic!();
            }
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

async fn recv_pkg(addr: std::net::SocketAddr, pkg_count: usize) {
    let mut socket = crate::generate_socket(addr, 4096 * 10, 4096); // parameter 8 

    let mut recv_pkg_count = 0usize;

    tokio::spawn(async move {
        let mut buffer = [0u8; 150];               // parameter 9 
        while recv_pkg_count < pkg_count {
            socket.recv(&mut buffer).await;
            recv_pkg_count += 1;

            if recv_pkg_count % 5 == 0 {
                println!("has recv: {}", recv_pkg_count);
            }
        }
        println!("mission complete");
    });
}
