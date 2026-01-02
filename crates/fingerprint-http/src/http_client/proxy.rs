//! proxysupport
//!
//! support HTTP and SOCKS5 proxy

use super::{HttpClientError, Result};
use std::io::{Read, Write};
use std::net::TcpStream;

/// proxytype
#[derive(Debug, Clone)]
pub enum ProxyType {
 /// HTTP proxy
 Http,
 /// HTTPS proxy
 Https,
 /// SOCKS5 proxy
 Socks5,
}

/// proxyconfiguration
#[derive(Debug, Clone)]
pub struct ProxyConfig {
 /// proxytype
 pub proxy_type: ProxyType,
 /// proxyserveraddress
 pub host: String,
 /// proxyserverport
 pub port: u16,
 /// user名（optional）
 pub username: Option<String>,
 /// cipher（optional）
 pub password: Option<String>,
}

impl ProxyConfig {
 /// Create HTTP proxyconfiguration
 pub fn http(host: String, port: u16) -> Self {
 Self {
 proxy_type: ProxyType::Http,
 host,
 port,
 username: None,
 password: None,
 }
 }

 /// Create SOCKS5 proxyconfiguration
 pub fn socks5(host: String, port: u16) -> Self {
 Self {
 proxy_type: ProxyType::Socks5,
 host,
 port,
 username: None,
 password: None,
 }
 }

 /// settingsauthenticationinfo
 pub fn with_auth(mut self, username: String, password: String) -> Self {
 self.username = Some(username);
 self.password = Some(password);
 self
 }
}

/// throughproxyconnection
pub fn connect_through_proxy(
 proxy: &ProxyConfig,
 target_host: &str,
 target_port: u16,
) -> Result<TcpStream> {
 match proxy.proxy_type {
 ProxyType::Http | ProxyType::Https => connect_http_proxy(proxy, target_host, target_port),
 ProxyType::Socks5 => connect_socks5_proxy(proxy, target_host, target_port),
 }
}

/// through HTTP proxyconnection
fn connect_http_proxy(
 proxy: &ProxyConfig,
 target_host: &str,
 target_port: u16,
) -> Result<TcpStream> {
 // connection to proxyserver
 let proxy_addr = format!("{}:{}", proxy.host, proxy.port);
 let mut stream = TcpStream::connect(&proxy_addr)
.map_err(|e| HttpClientError::ConnectionFailed(format!("connectionproxyfailure: {}", e)))?;

 // send CONNECT request
 let connect_request = format!(
 "CONNECT {}:{} HTTP/1.1\r\nHost: {}:{}\r\n\r\n",
 target_host, target_port, target_host, target_port
 );

 stream
.write_all(connect_request.as_bytes())
.map_err(HttpClientError::Io)?;

 // readresponse
 let mut buffer = vec![0u8; 1024];
 let n = stream.read(&mut buffer).map_err(HttpClientError::Io)?;

 let response = String::from_utf8_lossy(&buffer[..n]);

 // Checkresponsewhethersuccess
 if !response.contains("200") {
 return Err(HttpClientError::ConnectionFailed(format!(
 "proxyConnection failed: {}",
 response.lines().next().unwrap_or("not知error")
 )));
 }

 Ok(stream)
}

/// through SOCKS5 proxyconnection
fn connect_socks5_proxy(
 proxy: &ProxyConfig,
 target_host: &str,
 target_port: u16,
) -> Result<TcpStream> {
 // connection to proxyserver
 let proxy_addr = format!("{}:{}", proxy.host, proxy.port);
 let mut stream = TcpStream::connect(&proxy_addr)
.map_err(|e| HttpClientError::ConnectionFailed(format!("connectionproxyfailure: {}", e)))?;

 // SOCKS5 handshake
 // 1. sendauthenticationmethod
 let auth_methods = if proxy.username.is_some() {
 vec![0x05, 0x02, 0x00, 0x02] // version5，2method：无authentication and user名cipherauthentication
 } else {
 vec![0x05, 0x01, 0x00] // version5，1method：无authentication
 };

 stream
.write_all(&auth_methods)
.map_err(HttpClientError::Io)?;

 // 2. readserverselectmethod
 let mut response = [0u8; 2];
 stream
.read_exact(&mut response)
.map_err(HttpClientError::Io)?;

 if response[0] != 0x05 {
 return Err(HttpClientError::ConnectionFailed(
 "invalid SOCKS5 version".to_string(),
 ));
 }

 // 3. if needauthentication
 if response[1] == 0x02 {
 if let (Some(username), Some(password)) = (&proxy.username, &proxy.password) {
 let mut auth_request = vec![0x01]; // authenticationversion
 auth_request.push(username.len() as u8);
 auth_request.extend_from_slice(username.as_bytes());
 auth_request.push(password.len() as u8);
 auth_request.extend_from_slice(password.as_bytes());

 stream
.write_all(&auth_request)
.map_err(HttpClientError::Io)?;

 let mut auth_response = [0u8; 2];
 stream
.read_exact(&mut auth_response)
.map_err(HttpClientError::Io)?;

 if auth_response[1] != 0x00 {
 return Err(HttpClientError::ConnectionFailed(
 "SOCKS5 authenticationfailure".to_string(),
 ));
 }
 } else {
 return Err(HttpClientError::ConnectionFailed(
 "proxyneedauthenticationbutnotprovide凭data".to_string(),
 ));
 }
 } else if response[1] != 0x00 {
 return Err(HttpClientError::ConnectionFailed(format!(
 "不supportauthenticationmethod: {}",
 response[1]
 )));
 }

 // 4. sendconnectionrequest
 let mut connect_request = vec![
 0x05, // version
 0x01, // CONNECT 命令
 0x00, // preserve
 0x03, // domaintype
 ];
 connect_request.push(target_host.len() as u8);
 connect_request.extend_from_slice(target_host.as_bytes());
 connect_request.push((target_port >> 8) as u8);
 connect_request.push((target_port & 0xff) as u8);

 stream
.write_all(&connect_request)
.map_err(HttpClientError::Io)?;

 // 5. readconnectionresponse
 let mut connect_response = [0u8; 10]; // 至少10bytes
 stream
.read_exact(&mut connect_response[..4])
.map_err(HttpClientError::Io)?;

 if connect_response[1] != 0x00 {
 return Err(HttpClientError::ConnectionFailed(format!(
 "SOCKS5 Connection failed，error码: {}",
 connect_response[1]
 )));
 }

 // read剩余addressinfo
 match connect_response[3] {
 0x01 => {
 // IPv4
 stream
.read_exact(&mut connect_response[4..10])
.map_err(HttpClientError::Io)?;
 }
 0x03 => {
 // domain
 let mut len = [0u8; 1];
 stream.read_exact(&mut len).map_err(HttpClientError::Io)?;
 let mut addr = vec![0u8; len[0] as usize + 2];
 stream.read_exact(&mut addr).map_err(HttpClientError::Io)?;
 }
 0x04 => {
 // IPv6
 let mut addr = vec![0u8; 18];
 stream.read_exact(&mut addr).map_err(HttpClientError::Io)?;
 }
 _ => {
 return Err(HttpClientError::ConnectionFailed(
 "不supportaddresstype".to_string(),
 ));
 }
 }

 Ok(stream)
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_proxy_config() {
 let proxy = ProxyConfig::http("127.0.0.1".to_string(), 8080);
 assert_eq!(proxy.host, "127.0.0.1");
 assert_eq!(proxy.port, 8080);
 }

 #[test]
 fn test_proxy_with_auth() {
 let proxy = ProxyConfig::socks5("127.0.0.1".to_string(), 1080)
.with_auth("user".to_string(), "pass".to_string());
 assert_eq!(proxy.username, Some("user".to_string()));
 assert_eq!(proxy.password, Some("pass".to_string()));
 }
}
