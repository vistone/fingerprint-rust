#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Attempt to parse TLS handshake message
    if data.len() >= 4 {
        use fingerprint_tls::tls_handshake::TLSHandshake;
        let _ = TLSHandshake::from_bytes(data);
    }
    
    // Attempt to parse complete TLS record
    if data.len() >= 5 {
        use fingerprint_tls::tls_handshake::TLSRecord;
        let _ = TLSRecord::from_bytes(data);
    }
});
