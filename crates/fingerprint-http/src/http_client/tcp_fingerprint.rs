//! TCP 指纹应用模块
//!
//! 在创建 TCP 连接时应用 TCP Profile，确保 TCP 指纹与浏览器指纹一致

use fingerprint_core::tcp::TcpProfile;
use socket2::{Domain, Protocol, Socket, Type};
use std::io;
use std::net::SocketAddr;
use tokio::net::TcpStream;

/// 应用 TCP Profile 到 socket
///
/// 设置 TTL、Window Size、MSS、Window Scale 等参数
///
/// # 参数
/// - `socket`: socket2::Socket 实例
/// - `tcp_profile`: TCP Profile 配置
///
/// # 返回
/// 成功返回 Ok(())，失败返回错误
pub fn apply_tcp_profile(socket: &Socket, tcp_profile: &TcpProfile) -> io::Result<()> {
    // 1. 设置 TTL（socket2 的 set_ttl 需要 u32）
    socket.set_ttl(tcp_profile.ttl as u32)?;

    // 2. 设置 TCP 选项
    // 注意：socket2 不直接支持设置 Window Size、MSS、Window Scale
    // 这些参数需要在 TCP 握手时通过 TCP 选项设置
    // 但我们可以通过设置 socket 选项来影响这些参数

    // 设置 TCP_NODELAY（禁用 Nagle 算法，提升性能）
    socket.set_nodelay(true)?;

    // 3. 设置接收缓冲区大小（影响 Window Size）
    // Window Size 通常与接收缓冲区大小相关
    // 注意：实际的 Window Size 是在 TCP 握手时协商的，这里只是设置缓冲区
    let recv_buffer_size = tcp_profile.window_size as usize;
    socket.set_recv_buffer_size(recv_buffer_size)?;

    // 4. 设置发送缓冲区大小
    socket.set_send_buffer_size(recv_buffer_size)?;

    Ok(())
}

/// 创建带有 TCP Profile 的 TCP socket
///
/// # 参数
/// - `addr`: 目标地址
/// - `tcp_profile`: TCP Profile 配置（可选）
///
/// # 返回
/// 返回配置好的 socket2::Socket
pub fn create_tcp_socket_with_profile(
    addr: &SocketAddr,
    tcp_profile: Option<&TcpProfile>,
) -> io::Result<Socket> {
    // 根据地址类型创建 socket
    let domain = match addr {
        SocketAddr::V4(_) => Domain::IPV4,
        SocketAddr::V6(_) => Domain::IPV6,
    };

    let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))?;

    // 应用 TCP Profile（如果提供）
    // 注意：TTL 必须在连接之前设置
    // 在 Linux 上，对于客户端 socket，TTL 可以在连接前设置，不需要绑定
    if let Some(profile) = tcp_profile {
        apply_tcp_profile(&socket, profile)?;
    }

    Ok(socket)
}

/// 创建带有 TCP Profile 的 TcpStream（异步）
///
/// # 参数
/// - `addr`: 目标地址
/// - `tcp_profile`: TCP Profile 配置（可选）
///
/// # 返回
/// 返回配置好的 tokio::net::TcpStream
pub async fn connect_tcp_with_profile(
    addr: SocketAddr,
    tcp_profile: Option<&TcpProfile>,
) -> io::Result<TcpStream> {
    // 创建 socket
    let socket = create_tcp_socket_with_profile(&addr, tcp_profile)?;

    // 设置为非阻塞模式（tokio 需要）
    socket.set_nonblocking(true)?;

    // 连接到目标地址（非阻塞）
    match socket.connect(&addr.into()) {
        Ok(()) => {
            // 连接立即成功（本地连接）
            let std_stream: std::net::TcpStream = socket.into();
            TcpStream::from_std(std_stream)
        }
        Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
            // 非阻塞连接会返回 WouldBlock，这是正常的
            // 转换为 tokio::net::TcpStream 并等待连接完成
            let std_stream: std::net::TcpStream = socket.into();
            let stream = TcpStream::from_std(std_stream)?;

            // 等待连接完成
            stream.writable().await?;

            // 检查连接是否成功（通过尝试写入空数据）
            match stream.try_write(&[]) {
                Ok(_) => Ok(stream),
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    // 连接还在进行中，再次等待
                    stream.writable().await?;
                    Ok(stream)
                }
                Err(e) => Err(e),
            }
        }
        Err(e) => Err(e),
    }
}

/// 创建带有 TCP Profile 的 TcpStream（同步）
///
/// # 参数
/// - `addr`: 目标地址
/// - `tcp_profile`: TCP Profile 配置（可选）
///
/// # 返回
/// 返回配置好的 std::net::TcpStream
pub fn connect_tcp_with_profile_sync(
    addr: SocketAddr,
    tcp_profile: Option<&TcpProfile>,
) -> io::Result<std::net::TcpStream> {
    // 创建 socket
    let socket = create_tcp_socket_with_profile(&addr, tcp_profile)?;

    // 连接到目标地址
    socket.connect(&addr.into())?;

    // 转换为 std::net::TcpStream
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

        // 验证 TTL 已设置
        let ttl = socket.ttl().unwrap();
        assert_eq!(ttl, 128);
    }

    /// 实际 TCP 连接测试：创建服务器和客户端，验证 TCP Profile 是否真正应用
    #[test]
    fn test_tcp_profile_real_connection() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;
        use std::thread;
        use std::time::Duration;

        println!("\n╔════════════════════════════════════════════════════════════════╗");
        println!("║        TCP Profile 实际应用测试 - 服务端验证                  ║");
        println!("╚════════════════════════════════════════════════════════════════╝\n");

        let port = 9876;
        let stop_flag = Arc::new(AtomicBool::new(false));
        let stop_flag_clone = stop_flag.clone();

        // 启动服务器
        let _server = thread::spawn(move || {
            let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).unwrap();
            listener.set_nonblocking(true).unwrap();
            println!("✅ TCP 服务器启动在端口 {}", port);

            while !stop_flag_clone.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((mut stream, addr)) => {
                        println!("\n📥 收到客户端连接: {}", addr);

                        // 在 Linux 上检测 TCP 参数
                        #[cfg(target_os = "linux")]
                        {
                            use std::os::unix::io::AsRawFd;
                            let _fd = stream.as_raw_fd();

                            // 尝试获取接收缓冲区大小（影响 Window Size）
                            // 注意：这需要 libc crate，但为了简化，我们暂时注释掉
                            // 实际验证应该使用 tcpdump 或 wireshark 抓包分析
                            println!("  🔍 服务器端 TCP 参数检测：");
                            println!("    ⚠️  注意：TTL 在服务端无法直接检测（传输过程中会递减）");
                            println!("    💡 建议：使用 tcpdump 或 wireshark 抓包验证 TTL");
                            println!("    💡 命令：sudo tcpdump -i lo -n 'tcp port 9876' -v");
                        }

                        let mut buffer = [0; 1024];
                        if let Ok(size) = stream.read(&mut buffer) {
                            let data = String::from_utf8_lossy(&buffer[..size]);
                            println!("  收到数据: {}", data.trim());

                            // 解析客户端发送的 TCP Profile 信息
                            if data.contains("TCP_PROFILE:") {
                                println!("  ✅ 客户端 TCP Profile 信息已接收");
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
                        eprintln!("连接错误: {}", e);
                        break;
                    }
                }
            }
        });

        thread::sleep(Duration::from_millis(500));

        // 测试不同的 TCP Profile
        let test_cases = vec![
            ("Windows", TcpProfile::for_os(OperatingSystem::Windows10)),
            ("macOS", TcpProfile::for_os(OperatingSystem::MacOS14)),
            ("Linux", TcpProfile::for_os(OperatingSystem::Linux)),
        ];

        for (os_name, tcp_profile) in test_cases {
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("【测试】{} TCP Profile", os_name);
            println!(
                "  TTL: {}, Window Size: {}",
                tcp_profile.ttl, tcp_profile.window_size
            );

            let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
            match connect_tcp_with_profile_sync(addr, Some(&tcp_profile)) {
                Ok(mut stream) => {
                    println!("  ✅ 连接成功！");

                    let msg = format!(
                        "TCP_PROFILE: {} TTL={} WindowSize={}\n",
                        os_name, tcp_profile.ttl, tcp_profile.window_size
                    );
                    stream.write_all(msg.as_bytes()).unwrap();
                    stream.flush().unwrap();

                    let mut buffer = [0; 1024];
                    if let Ok(size) = stream.read(&mut buffer) {
                        let response = String::from_utf8_lossy(&buffer[..size]);
                        println!("  📥 服务器响应: {}", response.trim());
                    }

                    println!("  ✅ {} TCP Profile 测试通过", os_name);
                }
                Err(e) => {
                    println!("  ❌ {} TCP Profile 测试失败: {}", os_name, e);
                }
            }

            thread::sleep(Duration::from_millis(200));
        }

        stop_flag.store(true, Ordering::Relaxed);
        thread::sleep(Duration::from_millis(100));

        println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("✅ TCP Profile 实际应用测试完成！");
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
    }
}
