struct Distributor {
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

struct Sets {
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
struct Group{
    pub map:std::collections::HashMap<String, Sets>
}
impl Group{
    // 该函数用于从 Group 中获取初始化 lib.rs 的所有所需信息
    // 返回标记为 enable 的所有 Sets 中的所有 Distributor
    // 其中每个 Distributor 对应的 Vec 中只包含该 Distributor 中 enable 的 remote_addr
    pub fn get_plain_enable()->Vec<(std::net::SocketAddr, Vec<std::net::SocketAddr>)>{

    }
}
