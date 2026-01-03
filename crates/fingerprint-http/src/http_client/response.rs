//! HTTP response parsed 
//!
//! support：
//! - chunked encoding
//! - gzip/deflate/brotli compression
//! - complete HTTP/1.1 response parsed 

#[cfg(feature = "compression")]
use brotli_decompressor::Decompressor;
use std::collections::HashMap;
#[cfg(feature = "compression")]
use std::io::Read;

/// HTTP response
#[derive(Debug, Clone)]
pub struct HttpResponse {
 pub status_code: u16,
 pub status_text: String,
 pub headers: HashMap<String, String>,
 pub body: Vec<u8>,
 pub http_version: String,
 pub response_time_ms: u64, // response when between
}

impl HttpResponse {
 /// create a new response
 pub fn new(status_code: u16) -> Self {
 Self {
 status_code,
 status_text: String::new(),
 headers: HashMap::new(),
 body: Vec::new(),
 http_version: "HTTP/1.1".to_string(),
 response_time_ms: 0,
 }
 }

 /// from original beginningresponse parsed (completeversion)
 pub fn parse(raw_response: &[u8]) -> Result<Self, String> {
 let start = std::time::Instant::now();

 // 1. separate headers and body
 let (headers_end, body_start) = Self::find_headers_end(raw_response)?;

 let header_bytes = &raw_response[..headers_end];
 let body_bytes = &raw_response[body_start..];

 // 2. parsed headers
 let header_str = String::from_utf8_lossy(header_bytes);
 let mut lines = header_str.lines();

 // 3. parsed status row : HTTP/1.1 200 OK
 let status_line = lines.next().ok_or("missingstatus row ")?;
 let (http_version, status_code, status_text) = Self::parse_status_line(status_line)?;

 // 4. parsed headers
 let headers = Self::parse_headers(lines)?;

 // 5. process body
 let body = Self::process_body(body_bytes, &headers)?;

 let response_time_ms = start.elapsed().as_millis() as u64;

 Ok(Self {
 status_code,
 status_text,
 headers,
 body,
 http_version,
 response_time_ms,
 })
 }

 /// find headers endbit置 (\r\n\r\n)
 fn find_headers_end(data: &[u8]) -> Result<(usize, usize), String> {
 // securityCheck：ensurecountdatalengthat least as 4 bytes
 if data.len() < 4 {
 return Err("countdatatoo short，unable toincluding headers endmarker".to_string());
 }

 // use saturating_sub preventdown溢，butneed额outsideCheckedgeboundary
 let max_i = data.len().saturating_sub(3);
 for i in 0..max_i {
 // securityCheck：ensure not will 越boundaryaccess
 if i + 4 <= data.len() && &data[i..i + 4] == b"\r\n\r\n" {
 return Ok((i, i + 4));
 }
 }
 Err("not找 to headers endmarker".to_string())
 }

 /// parsed status row 
 fn parse_status_line(line: &str) -> Result<(String, u16, String), String> {
 let parts: Vec<&str> = line.splitn(3, ' ').collect();

 if parts.len() < 2 {
 return Err(format!("invalidstatus row : {}", line));
 }

 let http_version = parts[0].to_string();
 let status_code = parts[1]
.parse::<u16>()
.map_err(|_| format!("invalidstatus code: {}", parts[1]))?;
 let status_text = parts.get(2).unwrap_or(&"").to_string();

 Ok((http_version, status_code, status_text))
 }

 /// parsed headers
 fn parse_headers<'a, I>(lines: I) -> Result<HashMap<String, String>, String>
 where
 I: Iterator<Item = &'a str>,
 {
 let mut headers = HashMap::new();

 for line in lines {
 if line.is_empty() {
 continue;
 }

 if let Some(pos) = line.find(':') {
 let key = line[..pos].trim().to_lowercase(); // 转小写便于find
 let value = line[pos + 1..].trim().to_string();
 headers.insert(key, value);
 }
 }

 Ok(headers)
 }

 /// processresponse body (support chunked and compression)
 fn process_body(
 body_bytes: &[u8],
 headers: &HashMap<String, String>,
) -> Result<Vec<u8>, String> {
 let mut body = body_bytes.to_vec();

 // 1. process Transfer-Encoding: chunked
 if let Some(te) = headers.get("transfer-encoding") {
 if te.contains("chunked") {
 body = Self::parse_chunked(&body)?;
 }
 }

 // 2. process Content-Encoding
 if let Some(ce) = headers.get("content-encoding") {
 body = Self::decompress(&body, ce)?;
 }

 Ok(body)
 }

 /// parsed chunked encoding
 fn parse_chunked(data: &[u8]) -> Result<Vec<u8>, String> {
 /// maximum all ow single chunk size (10MB)
 /// preventmaliciousserversendoversized chunk causeinsidememory exhausted
 const MAX_CHUNK_SIZE: usize = 10 * 1024 * 1024; // 10MB

 let mut result = Vec::new();
 let mut pos = 0;

 loop {
 // find chunk size row end (\r\n)
 let size_line_end = data[pos..]
. window s(2)
.position(|w| w == b"\r\n")
.ok_or("Invalid chunked encoding: missing CRLF after size")?;

 // parsed chunk size (hexadecimal)
 let size_str = std::str::from_utf8(&data[pos..pos + size_line_end])
.map_err(|_| "Invalid chunk size: not UTF-8")?;

 // removemay's extensionsparameter (such as "3b; name=value")
 let size_str = size_str.split(';').next().unwrap_or(size_str).trim();

 let size = usize::from_str_radix(size_str, 16)
.map_err(|e| format!("Invalid chunk size '{}': {}", size_str, e))?;

 // securityCheck：preventmaliciousserversendoversized chunk
 if size > MAX_CHUNK_SIZE {
 return Err(format!(
 "Chunk size {} exceeds maximum all ow ed size {} bytes",
 size, MAX_CHUNK_SIZE
));
 }

 // size = 0 representlast chunk
 if size == 0 {
 break;
 }

 // skip size row and \r\n
 pos += size_line_end + 2;

 // Checkwhether have enoughcountdata
 if pos + size > data.len() {
 return Err(format!("Chunk size {} exceeds available data", size));
 }

 // Extract chunk data
 result.extend_from_slice(&data[pos..pos + size]);
 pos += size;

 // skip chunk back面 \r\n
 if pos + 2 <= data.len() && &data[pos..pos + 2] == b"\r\n" {
 pos += 2;
 } else {
 return Err("Invalid chunked encoding: missing CRLF after chunk data".to_string());
 }
 }

 Ok(result)
 }

 /// 解compressionresponse body 
 fn decompress(data: &[u8], encoding: &str) -> Result<Vec<u8>, String> {
 match encoding.to_lowercase().as_str() {
 #[cfg(feature = "compression")]
 "gzip" => Self::decompress_gzip(data),
 #[cfg(not(feature = "compression"))]
 "gzip" => Err("gzip decompressionneed --features compression".to_string()),
 #[cfg(feature = "compression")]
 "deflate" => Self::decompress_deflate(data),
 #[cfg(not(feature = "compression"))]
 "deflate" => Err("deflate decompressionneed --features compression".to_string()),
 "br" => Self::decompress_brotli(data),
 "identity" | "" => Ok(data.to_vec()),
 _ => Err(format!(" not supportencoding: {}", encoding)),
 }
 }

 /// decompression gzip
 #[cfg(feature = "compression")]
 fn decompress_gzip(data: &[u8]) -> Result<Vec<u8>, String> {
 #[cfg(not(feature = "compression"))]
 {
 let _ = data;
 return Err("gzip decompressionneedenabled feature: compression".to_string());
 }

 #[cfg(feature = "compression")]
 use flate2::read::GzDecoder;

 #[cfg(feature = "compression")]
 let mut decoder = GzDecoder::new(data);
 #[cfg(feature = "compression")]
 let mut result = Vec::new();
 #[cfg(feature = "compression")]
 decoder
.read_to_end(&mut result)
.map_err(|e| format!("gzip decompression failure: {}", e))?;
 #[cfg(feature = "compression")]
 Ok(result)
 }

 #[cfg(not(feature = "compression"))]
 fn decompress_gzip(_data: &[u8]) -> Result<Vec<u8>, String> {
 Err("compressionFeaturesnotenabled， please use --features compression compile".to_string())
 }

 /// decompression deflate
 #[cfg(feature = "compression")]
 fn decompress_deflate(data: &[u8]) -> Result<Vec<u8>, String> {
 #[cfg(not(feature = "compression"))]
 {
 let _ = data;
 return Err("deflate decompressionneedenabled feature: compression".to_string());
 }

 #[cfg(feature = "compression")]
 use flate2::read::DeflateDecoder;

 #[cfg(feature = "compression")]
 let mut decoder = DeflateDecoder::new(data);
 #[cfg(feature = "compression")]
 let mut result = Vec::new();
 #[cfg(feature = "compression")]
 decoder
.read_to_end(&mut result)
.map_err(|e| format!("deflate decompression failure: {}", e))?;
 #[cfg(feature = "compression")]
 Ok(result)
 }

 #[cfg(not(feature = "compression"))]
 fn decompress_deflate(_data: &[u8]) -> Result<Vec<u8>, String> {
 Err("compressionFeaturesnotenabled， please use --features compression compile".to_string())
 }

 /// decompression brotli
 #[cfg(feature = "compression")]
 fn decompress_brotli(data: &[u8]) -> Result<Vec<u8>, String> {
 let mut decompressor = Decompressor::new(data, 4096);
 let mut result = Vec::new();
 decompressor
.read_to_end(&mut result)
.map_err(|e| format!("brotli decompression failure: {}", e))?;
 Ok(result)
 }

 #[cfg(not(feature = "compression"))]
 fn decompress_brotli(_data: &[u8]) -> Result<Vec<u8>, String> {
 Err("brotli decompressionneedenabled feature: compression".to_string())
 }

 /// Getresponse body as string
 pub fn body_as_string(&self) -> Result<String, std::string::FromUtf8Error> {
 String::from_utf8(self.body.clone())
 }

 /// Checkwhethersuccess
 pub fn is_success(&self) -> bool {
 self.status_code >= 200 && self.status_code < 300
 }

 /// Get header
 pub fn get_header(&self, key: &str) -> Option<&String> {
 self.headers.get(key)
 }
}

#[cfg(test)]
mod tests {
 use super::*;
 #[cfg(feature = "compression")]
 use flate2::write::GzEncoder;
 #[cfg(feature = "compression")]
 use flate2::Compression;
 #[cfg(feature = "compression")]
 use std::io::Write;

 #[test]
 fn test_parse_response() {
 let raw =
 b"HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: 11\r\n\r\nHello World";

 let response = HttpResponse::parse(raw).unwrap();

 assert_eq!(response.status_code, 200);
 assert_eq!(response.status_text, "OK");
 // headers store when will convert to小写
 assert_eq!(
 response.get_header("content-type"),
 Some(&"text/html".to_string())
);
 assert_eq!(response.body_as_string().unwrap(), "Hello World");
 assert!(response.is_success());
 }

 #[test]
 fn test_parse_error_response() {
 let raw = b"HTTP/1.1 404 Not Found\r\n\r\n";

 let response = HttpResponse::parse(raw).unwrap();

 assert_eq!(response.status_code, 404);
 assert_eq!(response.status_text, "Not Found");
 assert!(!response.is_success());
 }

 #[test]
 fn test_chunked_encoding() {
 // Wiki (4)
 // pedia (5)
 // " in\r\n\r\nchunks." (14) -> E
 let raw = b"HTTP/1.1 200 OK\r\n\
 Transfer-Encoding: chunked\r\n\
 \r\n\
 4\r\n\
 Wiki\r\n\
 5\r\n\
 pedia\r\n\
 E\r\n\
 \x20in\r\n\
 \r\n\
 chunks.\r\n\
 0\r\n\
 \r\n";

 let response = HttpResponse::parse(raw).expect("Chunked parsed failure");
 assert_eq!(
 response.body_as_string().unwrap(),
 "Wikipedia in\r\n\r\nchunks."
);
 }

 #[test]
 #[cfg(feature = "compression")]
 fn test_gzip_compression() {
 let data = "Hello Gzip World";
 let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
 encoder.write_ all (data.as_bytes()).unwrap();
 let compressed = encoder.finish().unwrap();

 let mut raw = b"HTTP/1.1 200 OK\r\n\
 Content-Encoding: gzip\r\n\
 \r\n"
.to_vec();
 raw.extend_from_slice(&compressed);

 let response = HttpResponse::parse(&raw).expect("Gzip parsed failure");
 assert_eq!(response.body_as_string().unwrap(), data);
 }

 #[test]
 #[cfg(feature = "compression")]
 fn test_chunked_and_gzip() {
 let data = "Hello Chunked Gzip World";
 let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
 encoder.write_ all (data.as_bytes()).unwrap();
 let compressed = encoder.finish().unwrap();

 // will compressioncountdata分block
 let chunk1 = &compressed[0..10];
 let chunk2 = &compressed[10..];

 let mut raw = b"HTTP/1.1 200 OK\r\n\
 Transfer-Encoding: chunked\r\n\
 Content-Encoding: gzip\r\n\
 \r\n"
.to_vec();

 // Chunk 1
 raw.extend_from_slice(format!("{:x}\r\n", chunk1.len()).as_bytes());
 raw.extend_from_slice(chunk1);
 raw.extend_from_slice(b"\r\n");

 // Chunk 2
 raw.extend_from_slice(format!("{:x}\r\n", chunk2.len()).as_bytes());
 raw.extend_from_slice(chunk2);
 raw.extend_from_slice(b"\r\n");

 // Last chunk
 raw.extend_from_slice(b"0\r\n\r\n");

 let response = HttpResponse::parse(&raw).expect("Chunked+Gzip parsed failure");
 assert_eq!(response.body_as_string().unwrap(), data);
 }
}
