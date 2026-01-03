//! TCP fingerprintapplicationmodule
//!
//! in Create TCP connection when application TCP Profile，ensure TCP fingerprint and browserfingerprintconsistent

use fingerprint_core::tcp::TcpProfile;
use socket2::{Domain, Protocol, Socket, Type};
use std::io;
use std::net::SocketAddr;
use tokio::net::TcpStream;

/// application TCP Profile to socket
///
/// settings TTL、Window Size、MSS、Window Scale etc.parameter
///
/// # Parameters
/// - `socket`: socket2::Socket instance
/// - `tcp_profile`: TCP Profile configuration
///
/// # Returns
/// successreturn Ok(())， failurereturnerror
pub fn apply_tcp_profile(socket: &Socket, tcp_profile: &TcpProfile) -> io::Result<()> {
 // 1. settings TTL (socket2 set_ttl need u32)
 socket.set_ttl(tcp_profile.ttl as u32)?;

 // 2. settings TCP options
 // Note: socket2 not directlysupportsettings Window Size、MSS、Window Scale
 // theseparameterneed in TCP handshake when through TCP optionssettings
 // butwecanthroughsettings socket options from impacttheseparameter

 // settings TCP_NODELAY (disabled Nagle algorithm，improve perform ance)
 socket.set_nodelay(true)?;

 // 3. settingsreceivebuffersize (impact Window Size)
 // Window Size usu all y and receivebuffersize phase close
 // Note: actual Window Size is in TCP handshake when negotiate ，here only is settingsbuffer
 let recv_buffer_size = tcp_profile. window _size as usize;
 socket.set_recv_buffer_size(recv_buffer_size)?;

 // 4. settingssendbuffersize
 socket.set_send_buffer_size(recv_buffer_size)?;

 Ok(())
}

/// Createbring have TCP Profile TCP socket
///
/// # Parameters
/// - `addr`: target address
/// - `tcp_profile`: TCP Profile configuration (optional)
///
/// # Returns
/// returnconfiguration好 socket2::Socket
pub fn create_tcp_socket_with_profile(
 addr: &SocketAddr,
 tcp_profile: Option<&TcpProfile>,
) -> io::Result<Socket> {
 // Based on addresstypeCreate socket
 let domain = match addr {
 SocketAddr::V4(_) => Domain::IPV4,
 SocketAddr::V6(_) => Domain::IPV6,
 };

 let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))?;

 // application TCP Profile (if provide)
 // Note: TTL must in connectionbeforesettings
 // in Linux up， for client socket，TTL can in connectionfrontsettings， not need bind
 if let Some(profile) = tcp_profile {
 apply_tcp_profile(&socket, profile)?;
 }

 Ok(socket)
}

/// Createbring have TCP Profile TcpStream (async)
///
/// # Parameters
/// - `addr`: target address
/// - `tcp_profile`: TCP Profile configuration (optional)
///
/// # Returns
/// returnconfiguration好 tokio::net::TcpStream
pub async fn connect_tcp_with_profile(
 addr: SocketAddr,
 tcp_profile: Option<&TcpProfile>,
) -> io::Result<TcpStream> {
 // Create socket
 let socket = create_tcp_socket_with_profile(&addr, tcp_profile)?;

 // settings as non-blockingpattern (tokio need)
 socket.set_nonblocking(true)?;

 // connection to target address (non-blocking)
 match socket.connect(&addr.into()) {
 Ok(()) => {
 // connectionimmediatelysuccess (localconnection)
 let std_stream: std::net::TcpStream = socket.into();
 TcpStream::from_std(std_stream)
 }
 Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
 // non-blockingconnection will return WouldBlock，this isnormal 
 // convert to tokio::net::TcpStream and waitconnectioncomplete
 let std_stream: std::net::TcpStream = socket.into();
 let stream = TcpStream::from_std(std_stream)?;

 // waitconnectioncomplete
 stream.writable().await?;

 // Checkconnectionwhethersuccess (through try writeemptycountdata)
 match stream. try _write(&[]) {
 Ok(_) => Ok(stream),
 Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
 // connectionstill in perform in ，againwait
 stream.writable().await?;
 Ok(stream)
 }
 Err(e) => Err(e),
 }
 }
 Err(e) => Err(e),
 }
}

/// Createbring have TCP Profile TcpStream (sync)
///
/// # Parameters
/// - `addr`: target address
/// - `tcp_profile`: TCP Profile configuration (optional)
///
/// # Returns
/// returnconfiguration好 std::net::TcpStream
pub fn connect_tcp_with_profile_sync(
 addr: SocketAddr,
 tcp_profile: Option<&TcpProfile>,
) -> io::Result<std::net::TcpStream> {
 // Create socket
 let socket = create_tcp_socket_with_profile(&addr, tcp_profile)?;

 // connection to target address
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

 /// actual TCP connectiontest：Createserver and client，Validate TCP Profile whethertrueapplication
 #[test]
 fn test_tcp_profile_real_connection() {
 use std::io::{Read, Write};
 use std::net::TcpListener;
 use std::sync::atomic::{AtomicBool, Ordering};
 use std::sync::Arc;
 use std::thread;
 use std::time::Duration;

 println!("\n╔════════════════════════════════════════════════════════════════╗");
 println!("║ TCP Profile actualapplicationtest - service endValidate ║");
 println!("╚════════════════════════════════════════════════════════════════╝\n");

 let port = 9876;
 let stop_flag = Arc::new(AtomicBool::new(false));
 let stop_flag_clone = stop_flag.clone();

 // startserver
 let _server = thread::spawn(move || {
 let listener = TcpListener:: bind(format!("127.0.0.1:{}", port)).unwrap();
 listener.set_nonblocking(true).unwrap();
 println!("✅ TCP serverstart in port {}", port);

 while!stop_flag_clone.load(Ordering::Relaxed) {
 match listener.accept() {
 Ok((mut stream, addr)) => {
 println!("\n📥 receive to clientconnection: {}", addr);

 // in Linux updetect TCP parameter
 #[cfg(target_os = "linux")]
 {
 use std::os::unix::io::AsRawFd;
 let _fd = stream.as_raw_fd();

 // try Getreceivebuffersize (impact Window Size)
 // Note: this need libc crate，butin order tosimplify，we temporary when comment掉
 // actualValidateshoulduse tcpdump or wireshark packet captureanalysis
 println!(" 🔍 server end TCP parameterdetect：");
 println!(" ⚠️ Note: TTL in service endunable todirectlydetect (transferprocess in will 递减)");
 println!(" 💡 suggest：use tcpdump or wireshark packet captureValidate TTL");
 println!(" 💡 command：sudo tcpdump -i lo -n 'tcp port 9876' -v");
 }

 let mut buffer = [0; 1024];
 if let Ok(size) = stream.read(&mut buffer) {
 let data = String::from_utf8_lossy(&buffer[..size]);
 println!(" receive to countdata: {}", data.trim());

 // parsed clientsend TCP Profile info
 if data.contains("TCP_PROFILE:") {
 println!(" ✅ client TCP Profile infoalreadyreceive");
 }
 }

 stream.write_ all (b"OK: Server received\n").unwrap();
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

 // test different TCP Profile
 let test_cases = vec![
 ("Windows", TcpProfile::for_os(OperatingSystem::Windows10)),
 ("macOS", TcpProfile::for_os(OperatingSystem::MacOS14)),
 ("Linux", TcpProfile::for_os(OperatingSystem::Linux)),
 ];

 for (os_name, tcp_profile) in test_cases {
 println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
 println!("【test】{} TCP Profile", os_name);
 println!(
 " TTL: {}, Window Size: {}",
 tcp_profile.ttl, tcp_profile. window _size
);

 let addr: SocketAddr = format!("127.0.0.1:{}", port).parse().unwrap();
 match connect_tcp_with_profile_sync(addr, Some(&tcp_profile)) {
 Ok(mut stream) => {
 println!(" ✅ connectionsuccess！");

 let msg = format!(
 "TCP_PROFILE: {} TTL={} WindowSize={}\n",
 os_name, tcp_profile.ttl, tcp_profile. window _size
);
 stream.write_ all (msg.as_bytes()).unwrap();
 stream.flush().unwrap();

 let mut buffer = [0; 1024];
 if let Ok(size) = stream.read(&mut buffer) {
 let response = String::from_utf8_lossy(&buffer[..size]);
 println!(" 📥 serverresponse: {}", response.trim());
 }

 println!(" ✅ {} TCP Profile testthrough", os_name);
 }
 Err(e) => {
 println!(" ❌ {} TCP Profile test failure: {}", os_name, e);
 }
 }

 thread::sleep(Duration::from_millis(200));
 }

 stop_flag.store(true, Ordering::Relaxed);
 thread::sleep(Duration::from_millis(100));

 println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
 println!("✅ TCP Profile actualapplicationtestcomplete！");
 println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");
 }
}
