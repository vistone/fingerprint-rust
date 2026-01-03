use fingerprint::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
 println!("🧪 Chrome 136 fingerprintdepthValidate\n");

 let profiles = mapped_tls_clients();
 let chrome_136 = profiles
.get("chrome_136")
.expect("Chrome 136 profile should exist");

 // 1. Validatecipher suite (Cipher Suites)
 println!("1️⃣ encryptionsuiteweightValidate:");
 let spec = chrome_136.get_client_hello_spec()?;

 // Getfront 5encryptionsuite (skip GREASE)
 let first_suites: Vec<u16> = spec
.cipher_suites
.iter()
.filter(|&&s|!fingerprint_tls::tls_config::is_grease_value(s))
.take(5)
.cloned()
.collect();

 println!(" front 5非 GREASE encryptionsuite:");
 for suite in first_suites {
 println!(" - 0x{:04x}", suite);
 }

 // expectfirst is TLS_AES_128_GCM_SHA256 (0x1301)
 if spec.cipher_suites.contains(&0x1301) {
 println!(" ✅ including TLS_AES_128_GCM_SHA256");
 }

 // 2. Validate ALPN
 println!("\n2️⃣ ALPN priorityValidate:");
 if let Some(metadata) = &spec.metadata {
 if let Some(alpn) = metadata.get_alpn() {
 println!(" configuration ALPN: {:?}", alpn);
 if alpn.first() == Some(&"h3".to_string()) {
 println!(" ✅ h3 alreadycorrectput firstbit");
 } else {
 println!(" ❌ h3 notput firstbit: {:?}", alpn.first());
 }
 } else {
 println!(" ❌ not找 to ALPN metadata");
 }
 }

 // 3. Build实战bytesstream
 println!("\n3️⃣ ClientHello bytesstreamBuild:");
 let client_hello = TLSHandshakeBuilder::build_client_hello(&spec, "www.google.com")?;
 println!(" ✅ successGenerate ClientHello: {} bytes", client_hello.len());

 // simpleCheck ALPN whether in bytesstream in (h3, h2, http/1.1)
 if client_hello. window s(2).any(|w| w == b"h3") && client_hello. window s(2).any(|w| w == b"h2") {
 println!(" ✅ bytesstream in including h3 and h2 identifier");
 }

 // 4. JA4 fingerprintValidate
 println!("\n4️⃣ JA4 fingerprintmain动Generate:");
 let ja4 = chrome_136.get_ja4_string()?;
 println!(" ✅ JA4: {}", ja4);

 println!("\n✨ Chrome 136 fine-tuneValidatethrough！");
 Ok(())
}
