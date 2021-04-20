use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_func_get_flat_enable() {
        let dis_0 = Distributor {
            name: "dis_0".to_string(),
            note: "no comment".to_string(),
            enable: true,
            local_addr: "127.0.0.1:5503".parse().unwrap(),
            remote_addrs: vec![
                (
                    "127.0.0.1:19208".parse().unwrap(),
                    "no comment".to_string(),
                    true,
                ),
                (
                    "127.0.0.1:19209".parse().unwrap(),
                    "no comment".to_string(),
                    false,
                ),
                (
                    "127.0.0.1:19210".parse().unwrap(),
                    "no comment".to_string(),
                    true,
                ),
            ],
        };
        let dis_1 = Distributor {
            name: "dis_1".to_string(),
            note: "no comment".to_string(),
            enable: false,
            local_addr: "127.0.0.1:5504".parse().unwrap(),
            remote_addrs: vec![
                (
                    "127.0.0.1:19211".parse().unwrap(),
                    "no comment".to_string(),
                    true,
                ),
                (
                    "127.0.0.1:19212".parse().unwrap(),
                    "no comment".to_string(),
                    false,
                ),
                (
                    "127.0.0.1:19213".parse().unwrap(),
                    "no comment".to_string(),
                    true,
                ),
            ],
        };

        let mut set_0 = Sets {
            name: "sets_0".to_string(),
            note: "no comment".to_string(),
            enable: true,
            vec: vec![],
        };
        set_0.vec.push(dis_0);
        set_0.vec.push(dis_1);

        let dis_2 = Distributor {
            name: "dis_2".to_string(),
            note: "no comment".to_string(),
            enable: true,
            local_addr: "127.0.0.1:5505".parse().unwrap(),
            remote_addrs: vec![
                (
                    "127.0.0.1:19214".parse().unwrap(),
                    "no comment".to_string(),
                    true,
                ),
                (
                    "127.0.0.1:19215".parse().unwrap(),
                    "no comment".to_string(),
                    false,
                ),
                (
                    "127.0.0.1:19216".parse().unwrap(),
                    "no comment".to_string(),
                    true,
                ),
            ],
        };
        let dis_3 = Distributor {
            name: "dis_3".to_string(),
            note: "no comment".to_string(),
            enable: true,
            local_addr: "127.0.0.1:5506".parse().unwrap(),
            remote_addrs: vec![
                (
                    "127.0.0.1:19217".parse().unwrap(),
                    "no comment".to_string(),
                    false,
                ),
                (
                    "127.0.0.1:19218".parse().unwrap(),
                    "no comment".to_string(),
                    false,
                ),
                (
                    "127.0.0.1:19219".parse().unwrap(),
                    "no comment".to_string(),
                    true,
                ),
            ],
        };

        let mut set_1 = Sets {
            name: "sets_1".to_string(),
            note: "no comment".to_string(),
            enable: true,
            vec: vec![],
        };
        set_1.vec.push(dis_2);
        set_1.vec.push(dis_3);

        let mut group_0 = Group { vec: vec![] };
        group_0.vec.push(set_0);
        group_0.vec.push(set_1);

        println!("{:?}", group_0.get_flat_enable());
        println!("{:?}", group_0.get_plain_enable());

        let mut res = group_0.get_flat_enable();
        res.sort();

        group_0.save("params.json");

        assert_eq!(
            format!("{:?}",res),
            "[(127.0.0.1:5503, [127.0.0.1:19208, 127.0.0.1:19210]), (127.0.0.1:5505, [127.0.0.1:19214, 127.0.0.1:19216]), (127.0.0.1:5506, [127.0.0.1:19219])]"
        );
    }

    #[test]
    pub fn test_from_flat_enable() {
        let flat = vec![
            (
                "127.0.0.1:5503".parse().unwrap(),
                vec![
                    "127.0.0.1:19207".parse().unwrap(),
                    "127.0.0.1:19208".parse().unwrap(),
                ],
            ),
            (
                "127.0.0.1:5504".parse().unwrap(),
                vec![
                    "127.0.0.1:19209".parse().unwrap(),
                    "127.0.0.1:19210".parse().unwrap(),
                    "127.0.0.1:19211".parse().unwrap(),
                ],
            ),
        ];
        let res = Group::from_flat_enable(&flat);
        println!("{:?}", flat);
        println!("{:?}", res);
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Remote {
    pub addr: std::net::SocketAddr,
    pub note: String,
    pub enable: bool,
}
impl Remote {
    pub fn new(addr: std::net::SocketAddr, note: String, enable: bool) -> Self {
        Self { addr, note, enable }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Distributor {
    // key
    pub name: String,
    // notes
    pub note: String,
    pub enable: bool,
    pub recv_buffer: usize,
    // recv_point
    pub local_addr: std::net::SocketAddr,
    // (send_to_point, notes, enable_flag)
    pub remote_addrs: Vec<Remote>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Sets {
    pub name: String,
    pub note: String,
    pub enable: bool,
    pub vec: Vec<Distributor>,
}

// 需要被序列化和反序列化
// 需要增添相应命令行参数以满足 Group 需求
// 需要实现命令行参数 -> Group 的转化
// 命令行参数中添加 --save 用于表示保存当前命令行输入为 Group 序列化后文件
// 命令行参数中添加 --para 用于指定读取 Group 序列化后文件作为输入
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Group {
    pub send_buffer: usize,
    pub vec: Vec<Sets>,
}
impl Group {
    // 该函数用于从 Group 中获取初始化 lib.rs 的所有所需信息
    // 返回标记为 enable 的所有 Sets 中的所有 Distributor
    // 其中每个 Distributor 对应的 Vec 中只包含该 Distributor 中 enable 的 remote_addr
    pub fn get_flat_enable(&self) -> Vec<(usize, std::net::SocketAddr, Vec<std::net::SocketAddr>)> {
        self.vec
            .iter()
            .filter(|set| set.enable)
            .flat_map(|set| set.vec.iter())
            .filter(|distributor| distributor.enable)
            .map(|distributor| {
                (
                    distributor.recv_buffer,
                    distributor.local_addr,
                    distributor
                        .remote_addrs
                        .iter()
                        .filter(|remote| remote.enable)
                        .map(|remote| remote.addr)
                        .collect(),
                )
            })
            .collect()
    }

    pub fn from_flat_enable(
        flat: &Vec<(usize, std::net::SocketAddr, Vec<std::net::SocketAddr>)>,
        send_buffer: usize,
    ) -> Self {
        Group {
            send_buffer,
            vec: vec![Sets {
                name: "default_set".to_string(),
                note: "no comment".to_string(),
                enable: true,
                vec: flat
                    .iter()
                    .enumerate()
                    .map(|(index, (recv_buffer, local_addr, remote_addrs))| Distributor {
                        name: "dis_".to_string() + &index.to_string(),
                        note: "no comment".to_string(),
                        enable: true,
                        recv_buffer: *recv_buffer,
                        local_addr: *local_addr,
                        remote_addrs: remote_addrs
                            .iter()
                            .map(|addr| Remote::new(*addr, "no comment".to_string(), true))
                            .collect(),
                    })
                    .collect(),
            }],
        }
    }

    pub fn get_json(&self) -> Result<String, std::io::Error> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn save(&self, path: &str) -> Result<(), std::io::Error> {
        serde_json::to_writer(std::io::BufWriter::new(std::fs::File::create(path)?), self)?;
        Ok(())
    }

    pub fn load(path: &str) -> Result<Self, std::io::Error> {
        Ok(serde_json::from_reader(std::io::BufReader::new(
            std::fs::File::open(path)?,
        ))?)
    }

    // pub fn run(&self){

    //     match crate::initial(self.get_flat_enable(), cmd.recv_buffer, cmd.send_buffer, stop_trigger) {
    //         Ok((dis_vec, sender_map)) => {
    //             crate::run(dis_vec, sender_map).await;
    //             // recv_pkg("127.0.0.1:19208".parse().unwrap(), 100_000_0).await;
    //             // send_pkg("127.0.0.1:5503".parse().unwrap(), 100_000_0, 5e8).await;

    //             //stop_sender.send(());
    //         }
    //         Err(e) => {}
    //     }
    // }
}
