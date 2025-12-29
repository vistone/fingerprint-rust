//! IO 辅助：读取 HTTP/1.x 响应 bytes
//!
//! 目的：避免仅靠 `read_to_end()`（依赖连接关闭）导致的阻塞/等待问题。
//! 当前实现会：
//! - 先读到 `\r\n\r\n` 获取响应头
//! - 若有 `Content-Length`：读取到完整 body 后返回
//! - 若为 `Transfer-Encoding: chunked`：读取到 `0\r\n\r\n`（无 trailer 的常见场景）后返回
//! - 否则：读到 EOF（等价于连接关闭）
//!
//! 同时提供最大响应大小保护，防止内存被打爆。

use std::io;
use std::io::Read;

pub const DEFAULT_MAX_RESPONSE_BYTES: usize = 16 * 1024 * 1024; // 16MiB

fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() {
        return Some(0);
    }
    haystack.windows(needle.len()).position(|w| w == needle)
}

fn parse_headers_for_length_and_chunked(header_bytes: &[u8]) -> (Option<usize>, bool) {
    let header_str = String::from_utf8_lossy(header_bytes);
    let mut content_length: Option<usize> = None;
    let mut is_chunked = false;

    for line in header_str.lines().skip(1) {
        let (k, v) = match line.split_once(':') {
            Some((k, v)) => (k.trim(), v.trim()),
            None => continue,
        };

        if k.eq_ignore_ascii_case("content-length") {
            if let Ok(n) = v.parse::<usize>() {
                content_length = Some(n);
            }
        } else if k.eq_ignore_ascii_case("transfer-encoding")
            && v.to_ascii_lowercase().contains("chunked")
        {
            is_chunked = true;
        }
    }

    (content_length, is_chunked)
}

/// 读取 HTTP/1.x 响应原始 bytes（headers + body）
pub fn read_http1_response_bytes<R: Read>(reader: &mut R, max_bytes: usize) -> io::Result<Vec<u8>> {
    let mut buf: Vec<u8> = Vec::new();
    let mut tmp = [0u8; 8192];

    let mut headers_end: Option<usize> = None;
    let mut target_len: Option<usize> = None;
    let mut is_chunked = false;

    loop {
        if let Some(t) = target_len {
            if buf.len() >= t {
                break;
            }
        }

        if buf.len() >= max_bytes {
            return Err(io::Error::other(format!("响应过大（>{} bytes）", max_bytes)));
        }

        let n = reader.read(&mut tmp)?;
        if n == 0 {
            // EOF：连接关闭（或底层没更多数据）
            break;
        }
        buf.extend_from_slice(&tmp[..n]);

        // 解析 headers
        if headers_end.is_none() {
            if let Some(pos) = find_subsequence(&buf, b"\r\n\r\n") {
                let end = pos + 4;
                headers_end = Some(end);
                let (cl, chunked) = parse_headers_for_length_and_chunked(&buf[..end]);
                is_chunked = chunked;
                if let Some(cl) = cl {
                    target_len = Some(end.saturating_add(cl));
                }
            }
        }

        // chunked：常见无 trailer 的结束标记
        if is_chunked {
            if let Some(end) = headers_end {
                let body = &buf[end..];
                if find_subsequence(body, b"0\r\n\r\n").is_some() {
                    // 这里不尝试精确定位结束位置（trailer 情况较复杂），
                    // 只要读到结束标志即可返回，交给后续解析处理。
                    break;
                }
            }
        }
    }

    Ok(buf)
}
