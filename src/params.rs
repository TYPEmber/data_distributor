use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_func_get_flat_enable() {
        let dis_0 = Distributor {
            name: "dis_0".to_string(),
            notes: "no comment".to_string(),
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
            notes: "no comment".to_string(),
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
            notes: "no comment".to_string(),
            enable: true,
            map: std::collections::HashMap::new(),
        };
        set_0.map.insert(dis_0.name.clone(), dis_0);
        set_0.map.insert(dis_1.name.clone(), dis_1);

        let dis_2 = Distributor {
            name: "dis_2".to_string(),
            notes: "no comment".to_string(),
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
            notes: "no comment".to_string(),
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
            notes: "no comment".to_string(),
            enable: true,
            map: std::collections::HashMap::new(),
        };
        set_1.map.insert(dis_2.name.clone(), dis_2);
        set_1.map.insert(dis_3.name.clone(), dis_3);

        let mut group_0 = Group {
            map: std::collections::HashMap::new(),
        };
        group_0.map.insert(set_0.name.clone(), set_0);
        group_0.map.insert(set_1.name.clone(), set_1);

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
pub struct Distributor {
    // key
    pub name: String,
    // notes
    pub notes: String,
    pub enable: bool,
    // recv_point
    pub local_addr: std::net::SocketAddr,
    // (send_to_point, notes, enable_flag)
    pub remote_addrs: Vec<(std::net::SocketAddr, String, bool)>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Sets {
    pub name: String,
    pub notes: String,
    pub enable: bool,
    pub map: std::collections::HashMap<String, Distributor>,
}

// 需要被序列化和反序列化
// 需要增添相应命令行参数以满足 Group 需求
// 需要实现命令行参数 -> Group 的转化
// 命令行参数中添加 --save 用于表示保存当前命令行输入为 Group 序列化后文件
// 命令行参数中添加 --para 用于指定读取 Group 序列化后文件作为输入
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Group {
    pub map: std::collections::HashMap<String, Sets>,
}
impl Group {
    // 该函数用于从 Group 中获取初始化 lib.rs 的所有所需信息
    // 返回标记为 enable 的所有 Sets 中的所有 Distributor
    // 其中每个 Distributor 对应的 Vec 中只包含该 Distributor 中 enable 的 remote_addr
    pub fn get_flat_enable(&self) -> Vec<(std::net::SocketAddr, Vec<std::net::SocketAddr>)> {
        self.map
            .iter()
            .filter(|(_, set)| set.enable)
            .flat_map(|(_, set)| set.map.iter())
            .filter(|(_, distributor)| distributor.enable)
            .map(|(_, distributor)| {
                (
                    distributor.local_addr,
                    distributor
                        .remote_addrs
                        .iter()
                        .filter(|(_, _, enable)| *enable)
                        .map(|(addr, _, _)| *addr)
                        .collect(),
                )
            })
            .collect()
    }

    pub fn from_flat_enable(flat: &Vec<(std::net::SocketAddr, Vec<std::net::SocketAddr>)>) -> Self {
        let mut set_0 = Sets {
            name: "default_set".to_string(),
            notes: "no comment".to_string(),
            enable: true,
            map: std::collections::HashMap::new(),
        };

        flat.iter()
            .enumerate()
            .map(|(index, (local_addr, remote_addrs))| {
                set_0.map.insert(
                    "dis_".to_string() + &index.to_string(),
                    Distributor {
                        name: "dis_".to_string() + &index.to_string(),
                        notes: "no comment".to_string(),
                        enable: true,
                        local_addr: *local_addr,
                        remote_addrs: remote_addrs
                            .iter()
                            .map(|addr| (*addr, "no comment".to_string(), true))
                            .collect(),
                    },
                );
            })
            .count();

        let mut group = Group {
            map: std::collections::HashMap::new(),
        };
        group.map.insert(set_0.name.clone(), set_0);

        group
    }

    pub fn get_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }

    // TODO: error handle
    pub fn save(&self, path: &str) {
        let fs = std::io::BufWriter::new(std::fs::File::create(path).unwrap());
        serde_json::to_writer(fs, self).unwrap();
    }
    // TODO: error handle
    pub fn load(path: &str) -> Self {
        let fs = std::io::BufReader::new(std::fs::File::open(path).unwrap());
        serde_json::from_reader(fs).unwrap()
    }

    pub fn get_plain_enable(&self) -> Vec<(std::net::SocketAddr, Vec<std::net::SocketAddr>)> {
        let mut res = Vec::new();
        for (_, value) in self.map.iter() {
            if value.enable == true {
                for (_, value1) in value.map.iter() {
                    if value1.enable == true {
                        let mut res_remote = Vec::new();
                        for value2 in value1.remote_addrs.iter() {
                            if value2.2 == true {
                                res_remote.push(value2.0);
                            }
                        }
                        res.push((value1.local_addr, res_remote));
                    }
                }
            }
        }
        res
    }
}
