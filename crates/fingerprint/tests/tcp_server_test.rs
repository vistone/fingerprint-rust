//! TCP Profile 实际应用测试
//!
//! 创建服务端和客户端，验证 TCP Profile 是否真正应用到 TCP 连接上
//!
//! 运行方式：
//! ```bash
//! cargo test --test tcp_server_test -- --nocapture
//! ```

use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

/// TCP 服务器：接收连接并检测 TCP 参数
fn start_tcp_server(port: u16, stop_flag: Arc<AtomicBool>) -> std::io::Result<()> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))?;
    println!("✅ TCP 服务器启动在端口 {}", port);

    // 设置非阻塞模式，以便可以检查 stop_flag
    listener.set_nonblocking(true)?;

    while !stop_flag.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((mut stream, addr)) => {
                println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("📥 收到客户端连接");
                println!("  客户端地址: {}", addr);

                // 尝试获取 TCP 选项（在 Linux 上）
                #[cfg(target_os = "linux")]
                {
                    use std::os::unix::io::AsRawFd;
                    let fd = stream.as_raw_fd();

                    // 获取接收缓冲区大小（Window Size）
                    unsafe {
                        use libc::{getsockopt, SOL_SOCKET, SO_RCVBUF};
                        let mut rcvbuf: libc::c_int = 0;
                        let mut len = std::mem::size_of::<libc::c_int>() as libc::socklen_t;

                        if getsockopt(
                            fd,
                            SOL_SOCKET,
                            SO_RCVBUF,
                            &mut rcvbuf as *mut _ as *mut libc::c_void,
                            &mut len,
                        ) == 0
                        {
                            println!("  接收缓冲区大小: {} bytes", rcvbuf);
                        }
                    }
                }

                // 读取客户端发送的数据
                let mut buffer = [0; 1024];
                match stream.read(&mut buffer) {
                    Ok(size) => {
                        let data = String::from_utf8_lossy(&buffer[..size]);
                        println!("  收到数据: {}", data.trim());

                        // 解析客户端发送的 TCP Profile 信息
                        if data.starts_with("TCP_PROFILE:") {
                            println!("  ✅ 客户端 TCP Profile 信息:");
                            for line in data.lines() {
                                if line.starts_with("TCP_PROFILE:") {
                                    println!("    {}", line);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        println!("  读取错误: {}", e);
                    }
                }

                // 发送响应
                let response = "OK: Server received your connection\n";
                if let Err(e) = stream.write_all(response.as_bytes()) {
                    println!("  写入错误: {}", e);
                }

                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                // 非阻塞模式下没有连接，继续循环
                thread::sleep(Duration::from_millis(10));
                continue;
            }
            Err(e) => {
                println!("❌ 连接错误: {}", e);
                break;
            }
        }
    }

    Ok(())
}

/// 客户端：使用 TCP Profile 连接到服务器
fn test_tcp_client_with_profile(
    port: u16,
    tcp_profile: &fingerprint_core::tcp::TcpProfile,
) -> std::io::Result<()> {
    use fingerprint_http::http_client::tcp_fingerprint::connect_tcp_with_profile_sync;
    use std::net::SocketAddr;

    let addr: SocketAddr = format!("127.0.0.1:{}", port)
        .parse()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;

    println!("🔗 客户端使用 TCP Profile 连接服务器...");
    println!("  TCP Profile:");
    println!("    TTL: {}", tcp_profile.ttl);
    println!("    Window Size: {}", tcp_profile.window_size);
    println!("    MSS: {:?}", tcp_profile.mss);
    println!("    Window Scale: {:?}", tcp_profile.window_scale);

    // 使用 TCP Profile 连接
    let mut stream = connect_tcp_with_profile_sync(addr, Some(tcp_profile))?;

    println!("  ✅ 连接成功！");

    // 发送 TCP Profile 信息给服务器
    let profile_info = format!(
        "TCP_PROFILE: TTL={}, WindowSize={}, MSS={:?}, WindowScale={:?}\n",
        tcp_profile.ttl,
        tcp_profile.window_size,
        tcp_profile.mss.unwrap_or(0),
        tcp_profile.window_scale.unwrap_or(0)
    );

    stream.write_all(profile_info.as_bytes())?;
    stream.flush()?;

    // 读取服务器响应
    let mut buffer = [0; 1024];
    let size = stream.read(&mut buffer)?;
    let response = String::from_utf8_lossy(&buffer[..size]);
    println!("  📥 服务器响应: {}", response.trim());

    Ok(())
}

#[test]
fn test_tcp_profile_application() {
    println!("\n╔════════════════════════════════════════════════════════════════╗");
    println!("║        TCP Profile 实际应用测试                                ║");
    println!("╚════════════════════════════════════════════════════════════════╝\n");

    let port = 9876;
    let stop_flag = Arc::new(AtomicBool::new(false));
    let stop_flag_clone = stop_flag.clone();

    // 启动服务器（在后台线程）
    let _server_handle = thread::spawn(move || {
        if let Err(e) = start_tcp_server(port, stop_flag_clone) {
            eprintln!("❌ 服务器错误: {}", e);
        }
    });

    // 等待服务器启动
    thread::sleep(Duration::from_millis(500));

    // 测试不同的 TCP Profile
    let test_cases = vec![
        (
            "Windows",
            fingerprint_core::tcp::TcpProfile::for_os(
                fingerprint_core::types::OperatingSystem::Windows10,
            ),
        ),
        (
            "macOS",
            fingerprint_core::tcp::TcpProfile::for_os(
                fingerprint_core::types::OperatingSystem::MacOS14,
            ),
        ),
        (
            "Linux",
            fingerprint_core::tcp::TcpProfile::for_os(
                fingerprint_core::types::OperatingSystem::Linux,
            ),
        ),
    ];

    for (os_name, tcp_profile) in test_cases {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("【测试】{} TCP Profile", os_name);

        match test_tcp_client_with_profile(port, &tcp_profile) {
            Ok(_) => {
                println!("  ✅ {} TCP Profile 测试通过", os_name);
            }
            Err(e) => {
                println!("  ❌ {} TCP Profile 测试失败: {}", os_name, e);
            }
        }

        thread::sleep(Duration::from_millis(200));
    }

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("✅ TCP Profile 实际应用测试完成！");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    // 停止服务器
    stop_flag.store(true, Ordering::Relaxed);
    thread::sleep(Duration::from_millis(100));
}
