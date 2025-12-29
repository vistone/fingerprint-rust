//! 代理支持
//!
//! 支持 HTTP 和 SOCKS5 代理

use super::{HttpClientError, Result};
use std::io::{Read, Write};
use std::net::TcpStream;

/// 代理类型
#[derive(Debug, Clone)]
pub enum ProxyType {
    /// HTTP 代理
    Http,
    /// HTTPS 代理
    Https,
    /// SOCKS5 代理
    Socks5,
}

/// 代理配置
#[derive(Debug, Clone)]
pub struct ProxyConfig {
    /// 代理类型
    pub proxy_type: ProxyType,
    /// 代理服务器地址
    pub host: String,
    /// 代理服务器端口
    pub port: u16,
    /// 用户名（可选）
    pub username: Option<String>,
    /// 密码（可选）
    pub password: Option<String>,
}

impl ProxyConfig {
    /// 创建 HTTP 代理配置
    pub fn http(host: String, port: u16) -> Self {
        Self {
            proxy_type: ProxyType::Http,
            host,
            port,
            username: None,
            password: None,
        }
    }

    /// 创建 SOCKS5 代理配置
    pub fn socks5(host: String, port: u16) -> Self {
        Self {
            proxy_type: ProxyType::Socks5,
            host,
            port,
            username: None,
            password: None,
        }
    }

    /// 设置认证信息
    pub fn with_auth(mut self, username: String, password: String) -> Self {
        self.username = Some(username);
        self.password = Some(password);
        self
    }
}

/// 通过代理连接
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

/// 通过 HTTP 代理连接
fn connect_http_proxy(
    proxy: &ProxyConfig,
    target_host: &str,
    target_port: u16,
) -> Result<TcpStream> {
    // 连接到代理服务器
    let proxy_addr = format!("{}:{}", proxy.host, proxy.port);
    let mut stream = TcpStream::connect(&proxy_addr)
        .map_err(|e| HttpClientError::ConnectionFailed(format!("连接代理失败: {}", e)))?;

    // 发送 CONNECT 请求
    let connect_request = format!(
        "CONNECT {}:{} HTTP/1.1\r\nHost: {}:{}\r\n\r\n",
        target_host, target_port, target_host, target_port
    );

    stream
        .write_all(connect_request.as_bytes())
        .map_err(HttpClientError::Io)?;

    // 读取响应
    let mut buffer = vec![0u8; 1024];
    let n = stream.read(&mut buffer).map_err(HttpClientError::Io)?;

    let response = String::from_utf8_lossy(&buffer[..n]);

    // 检查响应是否成功
    if !response.contains("200") {
        return Err(HttpClientError::ConnectionFailed(format!(
            "代理连接失败: {}",
            response.lines().next().unwrap_or("未知错误")
        )));
    }

    Ok(stream)
}

/// 通过 SOCKS5 代理连接
fn connect_socks5_proxy(
    proxy: &ProxyConfig,
    target_host: &str,
    target_port: u16,
) -> Result<TcpStream> {
    // 连接到代理服务器
    let proxy_addr = format!("{}:{}", proxy.host, proxy.port);
    let mut stream = TcpStream::connect(&proxy_addr)
        .map_err(|e| HttpClientError::ConnectionFailed(format!("连接代理失败: {}", e)))?;

    // SOCKS5 握手
    // 1. 发送认证方法
    let auth_methods = if proxy.username.is_some() {
        vec![0x05, 0x02, 0x00, 0x02] // 版本5，2个方法：无认证和用户名密码认证
    } else {
        vec![0x05, 0x01, 0x00] // 版本5，1个方法：无认证
    };

    stream
        .write_all(&auth_methods)
        .map_err(HttpClientError::Io)?;

    // 2. 读取服务器选择的方法
    let mut response = [0u8; 2];
    stream
        .read_exact(&mut response)
        .map_err(HttpClientError::Io)?;

    if response[0] != 0x05 {
        return Err(HttpClientError::ConnectionFailed(
            "无效的 SOCKS5 版本".to_string(),
        ));
    }

    // 3. 如果需要认证
    if response[1] == 0x02 {
        if let (Some(username), Some(password)) = (&proxy.username, &proxy.password) {
            let mut auth_request = vec![0x01]; // 认证版本
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
                    "SOCKS5 认证失败".to_string(),
                ));
            }
        } else {
            return Err(HttpClientError::ConnectionFailed(
                "代理需要认证但未提供凭据".to_string(),
            ));
        }
    } else if response[1] != 0x00 {
        return Err(HttpClientError::ConnectionFailed(format!(
            "不支持的认证方法: {}",
            response[1]
        )));
    }

    // 4. 发送连接请求
    let mut connect_request = vec![
        0x05, // 版本
        0x01, // CONNECT 命令
        0x00, // 保留
        0x03, // 域名类型
    ];
    connect_request.push(target_host.len() as u8);
    connect_request.extend_from_slice(target_host.as_bytes());
    connect_request.push((target_port >> 8) as u8);
    connect_request.push((target_port & 0xff) as u8);

    stream
        .write_all(&connect_request)
        .map_err(HttpClientError::Io)?;

    // 5. 读取连接响应
    let mut connect_response = [0u8; 10]; // 至少10字节
    stream
        .read_exact(&mut connect_response[..4])
        .map_err(HttpClientError::Io)?;

    if connect_response[1] != 0x00 {
        return Err(HttpClientError::ConnectionFailed(format!(
            "SOCKS5 连接失败，错误码: {}",
            connect_response[1]
        )));
    }

    // 读取剩余的地址信息
    match connect_response[3] {
        0x01 => {
            // IPv4
            stream
                .read_exact(&mut connect_response[4..10])
                .map_err(HttpClientError::Io)?;
        }
        0x03 => {
            // 域名
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
                "不支持的地址类型".to_string(),
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
