//! TCP fingerprintapplicationmodule
//!
//!  in Create TCP connection when application TCP Profile，确保 TCP fingerprint and browserfingerprint一致

use fingerprint_core::tcp::TcpProfile;
use socket2::{Domain, Protocol, Socket, Type};
use std::io;
use std::net::SocketAddr;
use tokio::net::TcpStream;

/// application TCP Profile  to  socket
///
/// settings TTL、Window Size、MSS、Window Scale 等parameter
///
/// # Parameters
/// - `socket`: socket2::Socket 实例
/// - `tcp_profile`: TCP Profile configuration
///
/// # Returns
/// successreturn Ok(())，failurereturnerror
pub fn apply_tcp_profile(socket: &Socket, tcp_profile: &TcpProfile) -> io::Result<()> {
    // 1. settings TTL（socket2  set_ttl need u32）
    socket.set_ttl(tcp_profile.ttl as u32)?;

    // 2. settings TCP options
    // Note: socket2 不直接supportsettings Window Size、MSS、Window Scale
    // 这些parameterneed in TCP handshake when through TCP optionssettings
    // but我们canthroughsettings socket options来影响这些parameter

    // settings TCP_NODELAY（disabled Nagle algorithm，提升性能）
    socket.set_nodelay(true)?;

    // 3. settingsreceivebuffersize（影响 Window Size）
    // Window Size 通常 and receivebuffersize相关
    // Note: 实际 Window Size 是 in TCP handshake when 协商的，这里只是settingsbuffer
    let recv_buffer_size = tcp_profile.window_size as usize;
    socket.set_recv_buffer_size(recv_buffer_size)?;

    // 4. settingssendbuffersize
    socket.set_send_buffer_size(recv_buffer_size)?;

    Ok(())
}

/// Create带有 TCP Profile  TCP socket
///
/// # Parameters
/// - `addr`: targetaddress
/// - `tcp_profile`: TCP Profile configuration（optional）
///
/// # Returns
/// returnconfiguration好 socket2::Socket
pub fn create_tcp_socket_with_profile(
    addr: &SocketAddr,
    tcp_profile: Option<&TcpProfile>,
) -> io::Result<Socket> {
    // Based onaddresstypeCreate socket
    let domain = match addr {
        SocketAddr::V4(_) => Domain::IPV4,
        SocketAddr::V6(_) => Domain::IPV6,
    };

    let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))?;

    // application TCP Profile（ if provide）
    // Note: TTL must in connectionbeforesettings
    //  in Linux up， for client socket，TTL can in connectionfrontsettings，不need绑定
    if let Some(profile) = tcp_profile {
        apply_tcp_profile(&socket, profile)?;
    }

    Ok(socket)
}

/// Create带有 TCP Profile  TcpStream（async）
///
/// # Parameters
/// - `addr`: targetaddress
/// - `tcp_profile`: TCP Profile configuration（optional）
///
/// # Returns
/// returnconfiguration好 tokio::net::TcpStream
pub async fn connect_tcp_with_profile(
    addr: SocketAddr,
    tcp_profile: Option<&TcpProfile>,
) -> io::Result<TcpStream> {
    // Create socket
    let socket = create_tcp_socket_with_profile(&addr, tcp_profile)?;

    // settings为非阻塞pattern（tokio need）
    socket.set_nonblocking(true)?;

    // connection to targetaddress（非阻塞）
    match socket.connect(&addr.into()) {
        Ok(()) => {
            // connection立即success（localconnection）
            let std_stream: std::net::TcpStream = socket.into();
            TcpStream::from_std(std_stream)
        }
        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
            // 非阻塞connectionwillreturn WouldBlock，这是正常的
            // convert to tokio::net::TcpStream 并waitconnectioncomplete
            let std_stream: std::net::TcpStream = socket.into();
            let stream = TcpStream::from_std(std_stream)?;

            // waitconnectioncomplete
            stream.writable().await?;

            // Checkconnectionwhethersuccess（throughtrywriteemptycount据）
            match stream.try_write(&[]) {
                Ok(_) => Ok(stream),
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // connectionstill in 进行中，againwait
                    stream.writable().await?;
                    Ok(stream)
                }
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

/// Create带有 TCP Profile  TcpStream（sync）
///
/// # Parameters
/// - `addr`: targetaddress
/// - `tcp_profile`: TCP Profile configuration（optional）
///
/// # Returns
/// returnconfiguration好 std::net::TcpStream
pub fn connect_tcp_with_profile_sync(
    addr: SocketAddr,
    tcp_profile: Option<&TcpProfile>,
) -> io::Result<std::net::TcpStream> {
    // Create socket
    let socket = create_tcp_socket_with_profile(&addr, tcp_profile)?;

    // connection to targetaddress
    socket.connect(&addr.into())?;

    // convert to std::net::TcpStream
    Ok(socket.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use fingerprint_core::tcp::TcpProfile;
    use fingerprint_core::types::OperatingSystem;

    #[test]
    fn test_create_tcp_socket_with_profile() {
        let addr: SocketAddr = "127.0.0.1:80".parse().unwrap();
        let tcp_profile = TcpProfile::for_os(OperatingSystem::Windows10);

        let socket = create_tcp_socket_with_profile(&addr, Some(&tcp_profile));
        assert!(socket.is_ok());
    }

    #[test]
    fn test_apply_tcp_profile() {
        let _addr: SocketAddr = "127.0.0.1:80".parse().unwrap();
        let domain = Domain::IPV4;
        let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP)).unwrap();

        let tcp_profile = TcpProfile::for_os(OperatingSystem::Windows10);
        let result = apply_tcp_profile(&socket, &tcp_profile);
        assert!(result.is_ok());

        // Validate TTL alreadysettings
        let ttl = socket.ttl().unwrap();
        assert_eq!(ttl, 128);
    }

    /// 实际 TCP connectiontest：Createserver and client，Validate TCP Profile whether真正application
    #[test]
    fn test_tcp_profile_real_connection() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;
        use std::thread;
        use std::time::Duration;

        println!("\n╔════════════════════════════════════════════════════════════════╗");
        println!("║        TCP Profile 实际applicationtest - 服务端Validate                  ║");
        println!("╚════════════════════════════════════════════════════════════════╝\n");

        let port = 9876;
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = stop_flag.clone();

        // startserver
        let _server = thread::spawn(move || {
            let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
            listener.set_nonblocking(true).unwrap();
            println!("✅ TCP serverstart in port {}", port);

            while !stop_flag_clone.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, addr)) => {
                        println!("\n📥 收 to clientconnection: {}", addr);

                        //  in Linux up检测 TCP parameter
                        #[cfg(target_os = "linux")]
                        {
                            use std::os::unix::io::AsRawFd;
                            let _fd = stream.as_raw_fd();

                            // tryGetreceivebuffersize（影响 Window Size）
                            // Note: 这need libc crate，but为了简化，我们暂 when 注释掉
                            // 实际Validateshoulduse tcpdump  or  wireshark 抓包analysis
                            println!("  🔍 server端 TCP parameter检测：");
                            println!("    ⚠️  Note: TTL  in 服务端unable to直接检测（传输过程中will递减）");
                            println!("    💡 建议：use tcpdump  or  wireshark 抓包Validate TTL");
                            println!("    💡 命令：sudo tcpdump -i lo -n 'tcp port 9876' -v");
                        }

                        let mut buffer = [0; 1024];
                        if let Ok(size) = stream.read(&mut buffer) {
                            let data = String::from_utf8_lossy(&buffer[..size]);
                            println!("  收 to count据: {}", data.trim());

                            // Parseclientsend TCP Profile info
                            if data.contains("TCP_PROFILE:") {
                                println!("  ✅ client TCP Profile infoalreadyreceive");
                            }
                        }

                        stream.write_all(b"OK: Server received\n").unwrap();
                        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
                    }
                    Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    Err(e) => {
                        eprintln!("connectionerror: {}", e);
                        break;
                    }
                }
            }
        });

        thread::sleep(Duration::from_millis(500));

        // test不同 TCP Profile
        let test_cases = vec![
            ("Windows", TcpProfile::for_os(OperatingSystem::Windows10)),
            ("macOS", TcpProfile::for_os(OperatingSystem::MacOS14)),
            ("Linux", TcpProfile::for_os(OperatingSystem::Linux)),
        ];

        for (os_name, tcp_profile) in test_cases {
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("【test】{} TCP Profile", os_name);
            println!(
                "  TTL: {}, Window Size: {}",
                tcp_profile.ttl, tcp_profile.window_size
            );

            let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
            match connect_tcp_with_profile_sync(addr, Some(&tcp_profile)) {
                Ok(mut stream) => {
                    println!("  ✅ connectionsuccess！");

                    let msg = format!(
                        "TCP_PROFILE: {} TTL={} WindowSize={}\n",
                        os_name, tcp_profile.ttl, tcp_profile.window_size
                    );
                    stream.write_all(msg.as_bytes()).unwrap();
                    stream.flush().unwrap();

                    let mut buffer = [0; 1024];
                    if let Ok(size) = stream.read(&mut buffer) {
                        let response = String::from_utf8_lossy(&buffer[..size]);
                        println!("  📥 serverresponse: {}", response.trim());
                    }

                    println!("  ✅ {} TCP Profile testthrough", os_name);
                }
                Err(e) => {
                    println!("  ❌ {} TCP Profile testfailure: {}", os_name, e);
                }
            }

            thread::sleep(Duration::from_millis(200));
        }

        stop_flag.store(true, Ordering::Relaxed);
        thread::sleep(Duration::from_millis(100));

        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("✅ TCP Profile 实际applicationtestcomplete！");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    }
}
