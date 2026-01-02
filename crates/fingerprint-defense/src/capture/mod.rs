//! 数据包捕获模块
//!
//! 使用纯 Rust 实现从网络接口或文件实时捕获数据包（无系统依赖）。

use crate::passive::{PacketParser, PassiveAnalyzer};
use pnet::datalink::{self, Channel, NetworkInterface};
use std::sync::Arc;

/// 捕获引擎
pub struct CaptureEngine {
    analyzer: Arc<PassiveAnalyzer>,
}

impl CaptureEngine {
    /// 创建新的捕获引擎
    pub fn new(analyzer: Arc<PassiveAnalyzer>) -> Self {
        Self { analyzer }
    }

    /// 从指定网卡开始实时捕获
    pub async fn start_live(&self, device_name: &str) -> Result<(), String> {
        // 查找指定的网络接口
        let interface = datalink::interfaces()
            .into_iter()
            .find(|iface| iface.name == device_name)
            .ok_or_else(|| format!("找不到网络接口: {}", device_name))?;

        println!("[Capture] Listening on device: {}", device_name);

        let analyzer = self.analyzer.clone();

        // 使用 spawn_blocking 因为 pnet 的接收是阻塞的
        tokio::task::spawn_blocking(move || {
            Self::capture_from_interface(interface, analyzer)
        });

        Ok(())
    }

    /// 从网络接口捕获数据包（阻塞式）
    fn capture_from_interface(
        interface: NetworkInterface,
        analyzer: Arc<PassiveAnalyzer>,
    ) -> Result<(), String> {
        // 创建数据链路通道
        let (_tx, mut rx) = match datalink::channel(&interface, Default::default()) {
            Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
            Ok(_) => return Err("不支持的通道类型".to_string()),
            Err(e) => return Err(format!("创建通道失败: {}", e)),
        };

        // 循环接收数据包
        loop {
            match rx.next() {
                Ok(packet) => {
                    // 跳过以太网帧头（14 字节）
                    if packet.len() > 14 {
                        let ip_packet = &packet[14..];
                        if let Ok(p) = PacketParser::parse(ip_packet) {
                            let _ = analyzer.analyze(&p);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[Capture] 接收数据包错误: {}", e);
                    // 继续接收，不中断
                }
            }
        }
    }

    /// 从文件加载并处理
    pub fn process_file(&self, path: &str) -> Result<(), String> {
        use pcap_file::pcap::PcapReader;
        use std::fs::File;

        // 打开 pcap 文件
        let file = File::open(path).map_err(|e| format!("打开文件失败: {}", e))?;
        let mut pcap_reader =
            PcapReader::new(file).map_err(|e| format!("解析 pcap 文件失败: {}", e))?;

        // 读取所有数据包
        while let Some(packet) = pcap_reader.next_packet() {
            match packet {
                Ok(pkt) => {
                    // pcap 文件中的数据通常包含以太网帧头
                    let data = pkt.data;
                    if data.len() > 14 {
                        // 跳过以太网帧头（14 字节）
                        let ip_packet = &data[14..];
                        if let Ok(p) = PacketParser::parse(ip_packet) {
                            let _ = self.analyzer.analyze(&p);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[Capture] 读取数据包错误: {}", e);
                    // 继续处理下一个包
                }
            }
        }

        Ok(())
    }
}
