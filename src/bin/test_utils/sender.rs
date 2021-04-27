use data_distributor::*;

#[tokio::main]
async fn main() {
    recv_pkg("0.0.0.0:19208".parse().unwrap(), 100_000_0).await;
    send_pkg("127.0.0.1:5507".parse().unwrap(), 100_000_0, 9e8).await;

    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(1_000)).await;
    }
}

use socket2::{Domain, SockAddr, Socket, Type};

async fn send_pkg(addr: std::net::SocketAddr, pkg_count: usize, speed_rate: f64) {
    let any_addr: std::net::SocketAddr = "0.0.0.0:0".parse().unwrap();
    let mut socket = crate::generate_socket(any_addr, 4096, 4096 * 1024).unwrap();

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
            speed_now = send_bits_count as f64 / time_f64;
            //println!("{}", time.elapsed().unwrap().as_secs_f64());
            while (speed_now > speed_rate) {
                tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
                speed_now = send_bits_count as f64 / time.elapsed().unwrap().as_secs_f64()
            }
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
}
