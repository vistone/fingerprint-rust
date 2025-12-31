//! 数据包捕获模块
//!
//! 利用 pcap 库从网络接口或文件实时捕获数据包。

use crate::passive::{PacketParser, PassiveAnalyzer};
use pcap::{Capture, Device};
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
        let device = Device::from(device_name);
        let mut cap = Capture::from_device(device)
            .map_err(|e| e.to_string())?
            .promisc(true)
            .snaplen(65535)
            .timeout(1000)
            .open()
            .map_err(|e| e.to_string())?;

        println!("[Capture] Listening on device: {}", device_name);

        let analyzer = self.analyzer.clone();

        // 使用 spawn_blocking 因为 pcap 的 next() 是阻塞的
        tokio::task::spawn_blocking(move || {
            while let Ok(packet) = cap.next_packet() {
                if let Ok(p) = PacketParser::parse(packet.data) {
                    let _ = analyzer.analyze(&p);
                }
            }
        });

        Ok(())
    }

    /// 从文件加载并处理
    pub fn process_file(&self, path: &str) -> Result<(), String> {
        let mut cap = Capture::from_file(path).map_err(|e| e.to_string())?;

        while let Ok(packet) = cap.next_packet() {
            if let Ok(p) = PacketParser::parse(packet.data) {
                let _ = self.analyzer.analyze(&p);
            }
        }

        Ok(())
    }
}
