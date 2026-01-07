#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Attempt to parse HTTP response
    use fingerprint_http::http_client::HttpResponse;
    let _ = HttpResponse::parse(data);
    
    // Attempt to parse Set-Cookie header
    if let Ok(header_str) = std::str::from_utf8(data) {
        use fingerprint_http::http_client::Cookie;
        let _ = Cookie::parse_set_cookie(header_str, "example.com".to_string());
    }
});
