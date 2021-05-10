use data_distributor::*;

use structopt::StructOpt;

use log::{debug, error, info, trace, warn};

#[derive(StructOpt, Debug)]
pub struct Pair {
    /// local addr to listen
    //#[structopt(short, long)]
    local_addr: std::net::SocketAddr,

    /// remote addrs to send
    //#[structopt(short, long)]
    remote_addrs: Vec<std::net::SocketAddr>,
}
impl std::str::FromStr for Pair {
    type Err = std::net::AddrParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let addrs: Vec<&str> = s.split_whitespace().filter(|&x| x != "->").collect();

        let local = addrs[0].parse::<std::net::SocketAddr>()?;
        let mut remotes = vec![];
        for item in addrs[1..].into_iter() {
            remotes.push(item.parse::<std::net::SocketAddr>()?);
        }

        Ok(Pair {
            local_addr: local,
            remote_addrs: remotes,
        })
    }
}





#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(short, long, default_value = "1048576")]
    recv_buffer: usize,
    #[structopt(short, long, default_value = "4194304")]
    send_buffer: usize,
    #[structopt(short, long)]
    add: Vec<Pair>,
    #[structopt(long)]
    save: bool,
    #[structopt(long, default_value = "8080")]
    server: u16,
    #[structopt(long)]
    load_disable: bool,
    #[structopt(long, default_value="./params.json")]
    para: String,
}

// cargo run --bin main --release -- -a "127.0.0.1:5503 -> 127.0.0.1:19208 127.0.0.1:19210" -a "127.0.0.1:5504 -> 127.0.0.1:19211 127.0.0.1:19212"
#[tokio::main]
async fn main() {
    let cmd = Opt::from_args();
    let (stop_trigger, _) = tokio::sync::broadcast::channel(1);
    crate::logger::init();

    let recv_buffer = cmd.recv_buffer;
    let send_buffer = cmd.send_buffer;

    let (dis_vec, group) = if let false = cmd.load_disable {
        let group = params::Group::load(&cmd.para[..]).unwrap();

        (group.get_flat_enable(), group)
    } else {
        let dis_vec = cmd
            .add
            .into_iter()
            .map(|p| (recv_buffer, p.local_addr, p.remote_addrs))
            .collect();
        let group = params::Group::from_flat_enable(&dis_vec, send_buffer);

        if cmd.save {
            group.save("params.json").unwrap();
        }

        (dis_vec, group)
    };

    let msg_rx = crate::logger::subscribe();
    info!("GROUP {}", group.get_json().unwrap());

    match crate::initial(dis_vec, group.send_buffer, stop_trigger.clone()) {
        Ok((dis_vec, sender_map)) => {
            crate::run(dis_vec, sender_map).await;
            // recv_pkg("127.0.0.1:19208".parse().unwrap(), 100_000_0).await;
            // send_pkg("127.0.0.1:5503".parse().unwrap(), 100_000_0, 1e9).await;

            //stop_sender.send(());
        }
        Err(e) => warn!("{}", e),
    }

    crate::server::run(cmd.server, msg_rx, stop_trigger.clone()).await;
}

async fn test_socket_send_limited() {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(5_000));
    let loop_times = 200_000_0;

    let one_socket_cost = tokio::spawn(async move {
        let socket = generate_socket("0.0.0.0:0".parse().unwrap(), 1024, 1024 * 1024 * 10).unwrap();
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
        let socket = generate_socket("0.0.0.0:0".parse().unwrap(), 1024, 1024 * 1024 * 10).unwrap();
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
