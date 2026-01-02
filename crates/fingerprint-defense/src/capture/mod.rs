//! count据包捕获module
//!
//! use纯 Rust implement from networkinterface or file实 when 捕获count据包（无system依赖）。

use crate::passive::{PacketParser, PassiveAnalyzer};
use pnet::datalink::{self, Channel, NetworkInterface};
use std::sync::Arc;

/// 捕获引擎
pub struct CaptureEngine {
    analyzer: Arc<PassiveAnalyzer>,
}

impl CaptureEngine {
    /// Create a new捕获引擎
    pub fn new(analyzer: Arc<PassiveAnalyzer>) -> Self {
        Self { analyzer }
    }

    ///  from specified网卡start实 when 捕获
    pub async fn start_live(&self, device_name: &str) -> Result<(), String> {
        // 查找specified的networkinterface
        let interface = datalink::interfaces()
            .into_iter()
            .find(|iface| iface.name == device_name)
            .ok_or_else(|| format!("找不 to networkinterface: {}", device_name))?;

        println!("[Capture] Listening on device: {}", device_name);

        let analyzer = self.analyzer.clone();

        // use spawn_blocking because pnet 的receive是阻塞的
        tokio::task::spawn_blocking(move || {
            Self::capture_from_interface(interface, analyzer)
        });

        Ok(())
    }

    ///  from networkinterface捕获count据包（阻塞式）
    fn capture_from_interface(
        interface: NetworkInterface,
        analyzer: Arc<PassiveAnalyzer>,
    ) -> Result<(), String> {
        // Createcount据链路通道
        let (_tx, mut rx) = match datalink::channel(&interface, Default::default()) {
            Ok(Channel::Ethernet(tx, rx)) => (tx, rx),
            Ok(_) => return Err("不support的通道type".to_string()),
            Err(e) => return Err(format!("Create通道failure: {}", e)),
        };

        // 循环receivecount据包
        loop {
            match rx.next() {
                Ok(packet) => {
                    // securityCheck：limitmaximumcount据包size以防止 DoS 攻击（65535 bytes = maximum IP 包）
                    const MAX_PACKET_SIZE: usize = 65535;
                    if packet.len() > MAX_PACKET_SIZE {
                        eprintln!("[Capture] count据包过大，already忽略: {} bytes", packet.len());
                        continue;
                    }
                    
                    // skip以太网frameheader（14 bytes）
                    if packet.len() > 14 {
                        let ip_packet = &packet[14..];
                        if let Ok(p) = PacketParser::parse(ip_packet) {
                            let _ = analyzer.analyze(&p);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[Capture] receivecount据包error: {}", e);
                    // continuereceive，不中断
                }
            }
        }
    }

    ///  from fileload并process
    pub fn process_file(&self, path: &str) -> Result<(), String> {
        use pcap_file::pcap::PcapReader;
        use std::fs::File;

        // open pcap file
        let file = File::open(path).map_err(|e| format!("openfilefailure: {}", e))?;
        let mut pcap_reader =
            PcapReader::new(file).map_err(|e| format!("Parse pcap filefailure: {}", e))?;

        // readallcount据包
        let mut packet_count = 0;
        const MAX_PACKETS: usize = 1_000_000; // limitmaximumcount据包count以防止inside存耗尽
        
        while let Some(packet) = pcap_reader.next_packet() {
            // securityCheck：limitprocess的count据包count
            packet_count += 1;
            if packet_count > MAX_PACKETS {
                eprintln!("[Capture] already达 to maximumcount据包processlimit: {}", MAX_PACKETS);
                break;
            }
            
            match packet {
                Ok(pkt) => {
                    // securityCheck：limitsinglecount据包size
                    const MAX_PACKET_SIZE: usize = 65535;
                    let data = pkt.data;
                    if data.len() > MAX_PACKET_SIZE {
                        eprintln!("[Capture] count据包过大，already忽略: {} bytes", data.len());
                        continue;
                    }
                    
                    // pcap file中的count据通常including以太网frameheader
                    if data.len() > 14 {
                        // skip以太网frameheader（14 bytes）
                        let ip_packet = &data[14..];
                        if let Ok(p) = PacketParser::parse(ip_packet) {
                            let _ = self.analyzer.analyze(&p);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("[Capture] readcount据包error: {}", e);
                    // continueprocessnext包
                }
            }
        }

        println!("[Capture] alreadyprocess {} 个count据包", packet_count);
        Ok(())
    }
}
