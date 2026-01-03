//! IO auxiliary：read HTTP/1.x response bytes
//!
//! destination：avoidonly靠 `read_to_end()` (dependconnectionclose)causeblocking/waitissue。
//! currentimplementwill：
//! - read first to `\r\n\r\n` Getresponseheader
//! - 若有 `Content-Length`：read to complete body backreturn
//! - 若 as `Transfer-Encoding: chunked`：read to `0\r\n\r\n` (none trailer commonscenario)backreturn
//! - otherwise：读 to EOF ( etc.价于connectionclose)
//!
//! same when providemaximumresponsesizeprotect，preventinsidesave被打爆。

use std::io;
use std::io::Read;

pub const DEFAULT_MAX_RESPONSE_BYTES: usize = 16 * 1024 * 1024; // 16MiB
/// maximumallow Content-Length value (100MB)
/// preventmaliciousserversendoversized Content-Length causeinsidememory exhausted
pub const MAX_CONTENT_LENGTH: usize = 100 * 1024 * 1024; // 100MB

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

/// read HTTP/1.x responseoriginalbeginning bytes (headers + body)
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
 return Err(io::Error::other(format!(
 "responsetoo large (>{} bytes)",
 max_bytes
 )));
 }

 let n = reader.read(&mut tmp)?;
 if n == 0 {
 // EOF：connectionclose ( or bottomlayer没morecountdata)
 break;
 }
 buf.extend_from_slice(&tmp[..n]);

 // Parse headers
 if headers_end.is_none() {
 if let Some(pos) = find_subsequence(&buf, b"\r\n\r\n") {
 let end = pos + 4;
 headers_end = Some(end);
 let (cl, chunked) = parse_headers_for_length_and_chunked(&buf[..end]);
 is_chunked = chunked;
 if let Some(cl) = cl {
 // securityCheck：preventmaliciousserversendoversized Content-Length
 if cl > MAX_CONTENT_LENGTH {
 return Err(io::Error::other(format!(
 "Content-Length too large: {} bytes (maximum: {} bytes)",
 cl, MAX_CONTENT_LENGTH
 )));
 }
 target_len = Some(end.saturating_add(cl));
 }
 }
 }

 // chunked：commonnone trailer endmarker
 if is_chunked {
 if let Some(end) = headers_end {
 let body = &buf[end..];
 if find_subsequence(body, b"0\r\n\r\n").is_some() {
 // here不tryprecisedeterminebitendbitplace (trailer situationcomparecomplex)，
 // as long as读 to endflagcanreturn，交给backcontinueParseprocess。
 break;
 }
 }
 }
 }

 Ok(buf)
}
