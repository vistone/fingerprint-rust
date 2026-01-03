//! JA3/JA3S TLS fingerprintimplement
//!
//! JA3 is Salesforce open 发 TLS clientfingerprint identifymethod，alreadybecome row 业standard。
//! JA3S is pair should server endfingerprint。
//!
//! ## reference
//! - paper: "TLS Fingerprinting with JA3 and JA3S" (Salesforce, 2017)
//! - GitHub: https://github.com/salesforce/ja3

use serde::{Deserialize, Serialize};

/// JA3 TLS clientfingerprint
///
/// format: MD5(SSLVersion,Ciphers,Extensions,EllipticCurves,EllipticCurvePointFormats)
///
/// ## Examples
/// ```
/// use fingerprint_core::ja3::JA3;
///
/// let ja3 = JA3::generate(
/// 771, // TLS 1.2
/// &[0x1301, 0x1302, 0x1303],
/// &[0, 10, 11, 13],
/// &[23, 24, 25],
/// &[0],
///);
/// assert!(!ja3.fingerprint.is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JA3 {
 /// SSL/TLS version (decimal)
 pub ssl_version: u16,
 
 /// cipher suitelist (comma-separated decimal)
 pub ciphers: String,
 
 /// extensionlist (comma-separated decimal)
 pub extensions: String,
 
 /// elliptic curvelist (comma-separated decimal)
 pub elliptic_curves: String,
 
 /// elliptic curve point formatlist (comma-separated decimal)
 pub ec_point_formats: String,
 
 /// complete JA3 string (for Calculatehash)
 pub ja3_string: String,
 
 /// JA3 fingerprint (MD5 hash)
 pub fingerprint: String,
}

impl JA3 {
 /// Generate JA3 fingerprint
 ///
 /// # Parameters
 /// - `ssl_version`: TLS version (for example：771 = TLS 1.2, 772 = TLS 1.3)
 /// - `ciphers`: cipher suitelist (hexadecimalvalue)
 /// - `extensions`: extensionlist (hexadecimalvalue)
 /// - `elliptic_curves`: elliptic curvelist (hexadecimalvalue)
 /// - `ec_point_formats`: elliptic curve point formatlist (hexadecimalvalue)
 ///
 /// # Returns
 /// JA3 fingerprintstruct
 pub fn generate(
 ssl_version: u16,
 ciphers: &[u16],
 extensions: &[u16],
 elliptic_curves: &[u16],
 ec_point_formats: &[u8],
) -> Self {
 // filter GREASE value (if need)
 let filtered_ciphers: Vec<u16> = ciphers
.iter()
.filter(|&&c|!crate::grease::is_grease_value(c))
.cloned()
.collect();

 let filtered_extensions: Vec<u16> = extensions
.iter()
.filter(|&&e|!crate::grease::is_grease_value(e))
.cloned()
.collect();

 let filtered_curves: Vec<u16> = elliptic_curves
.iter()
.filter(|&&c|!crate::grease::is_grease_value(c))
.cloned()
.collect();

 // convert tocomma-separated decimalstring (JA3 usedecimal，is nothexadecimal)
 let ciphers_str = filtered_ciphers
.iter()
.map(|c| c.to_string())
.collect::<Vec<String>>()
.join("-");

 let extensions_str = filtered_extensions
.iter()
.map(|e| e.to_string())
.collect::<Vec<String>>()
.join("-");

 let curves_str = filtered_curves
.iter()
.map(|c| c.to_string())
.collect::<Vec<String>>()
.join("-");

 let formats_str = ec_point_formats
.iter()
.map(|f| f.to_string())
.collect::<Vec<String>>()
.join("-");

 // Build JA3 string
 let ja3_string = format!(
 "{},{},{},{},{}",
 ssl_version, ciphers_str, extensions_str, curves_str, formats_str
);

 // Calculate MD5 hash
 let fingerprint = Self::md5_hash(&ja3_string);

 Self {
 ssl_version,
 ciphers: ciphers_str,
 extensions: extensions_str,
 elliptic_curves: curves_str,
 ec_point_formats: formats_str,
 ja3_string,
 fingerprint,
 }
 }

 /// Calculate MD5 hash
 fn md5_hash(input: &str) -> String {
 // MD5 computation
 let digest = md5::compute(input.as_bytes());
 // Computed above
 format!("{:x}", digest)
 }

 /// from ClientHello original beginningcountdataGenerate JA3
 ///
 /// this isan便捷method， for from complete ClientHello message in Extract and Generate JA3
 pub fn from_client_hello(client_hello: &crate::signature::ClientHelloSignature) -> Self {
 // Convertelliptic curve CurveID as u16
 let curves: Vec<u16> = client_hello
.elliptic_curves
.iter()
.map(|c| *c as u16)
.collect();

 Self::generate(
 client_hello.version.to_u16(),
 &client_hello.cipher_suites,
 &client_hello.extensions,
 &curves,
 &client_hello.elliptic_curve_point_formats,
)
 }
}

impl std::fmt::Display for JA3 {
 fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
 write!(f, "{}", self.fingerprint)
 }
}

/// JA3S TLS serverfingerprint
///
/// format: MD5(SSLVersion,Cipher,Extensions)
///
/// ## Examples
/// ```
/// use fingerprint_core::ja3::JA3S;
///
/// let ja3s = JA3S::generate(771, 0x1301, &[0, 10, 11]);
/// assert!(!ja3s.fingerprint.is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JA3S {
 /// SSL/TLS version (decimal)
 pub ssl_version: u16,
 
 /// select 's cipher suites (decimal)
 pub cipher: u16,
 
 /// extensionlist (comma-separated decimal)
 pub extensions: String,
 
 /// complete JA3S string (for Calculatehash)
 pub ja3s_string: String,
 
 /// JA3S fingerprint (MD5 hash)
 pub fingerprint: String,
}

impl JA3S {
 /// Generate JA3S fingerprint
 ///
 /// # Parameters
 /// - `ssl_version`: TLS version
 /// - `cipher`: server select 's cipher suites
 /// - `extensions`: serverreturn's extensionslist
 pub fn generate(ssl_version: u16, cipher: u16, extensions: &[u16]) -> Self {
 // filter GREASE value
 let filtered_extensions: Vec<u16> = extensions
.iter()
.filter(|&&e|!crate::grease::is_grease_value(e))
.cloned()
.collect();

 // convert tocomma-separated decimalstring
 let extensions_str = filtered_extensions
.iter()
.map(|e| e.to_string())
.collect::<Vec<String>>()
.join("-");

 // Build JA3S string
 let ja3s_string = format!("{},{},{}", ssl_version, cipher, extensions_str);

 // Calculate MD5 hash
 let fingerprint = Self::md5_hash(&ja3s_string);

 Self {
 ssl_version,
 cipher,
 extensions: extensions_str,
 ja3s_string,
 fingerprint,
 }
 }

 /// Calculate MD5 hash
 fn md5_hash(input: &str) -> String {
 // MD5 computation
 let digest = md5::compute(input.as_bytes());
 // Computed above
 format!("{:x}", digest)
 }
}

impl std::fmt::Display for JA3S {
 fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
 write!(f, "{}", self.fingerprint)
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_ja3_generation() {
 // test Chrome browsertypical ClientHello
 let ja3 = JA3::generate(
 771, // TLS 1.2
 &[0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f],
 &[0, 10, 11, 13, 16, 23],
 &[23, 24, 25],
 &[0],
);

 assert!(!ja3.fingerprint.is_empty());
 assert_eq!(ja3.fingerprint.len(), 32); // MD5 hashlength
 assert_eq!(ja3.ssl_version, 771);
 }

 #[test]
 fn test_ja3_with_grease() {
 // testincluding GREASE valuesituation
 let ja3 = JA3::generate(
 771,
 &[0x0a0a, 0x1301, 0x1a1a], // including GREASE
 &[0x0a0a, 0, 10], // including GREASE
 &[0x0a0a, 23], // including GREASE
 &[0],
);

 // GREASE valueshould by filter掉
 assert!(!ja3.ciphers.contains("2570")); // 0x0a0a = 2570
 assert!(!ja3.extensions.contains("2570"));
 }

 #[test]
 fn test_ja3_empty_fields() {
 // testemptyfield
 let ja3 = JA3::generate(771, &[], &[], &[], &[]);

 assert!(!ja3.fingerprint.is_empty());
 assert_eq!(ja3.ciphers, "");
 assert_eq!(ja3.extensions, "");
 }

 #[test]
 fn test_ja3_display() {
 let ja3 = JA3::generate(771, &[0x1301], &[0], &[23], &[0]);
 let displayed = format!("{}", ja3);
 assert_eq!(displayed, ja3.fingerprint);
 }

 #[test]
 fn test_ja3s_generation() {
 let ja3s = JA3S::generate(771, 0x1301, &[0, 10, 11]);

 assert!(!ja3s.fingerprint.is_empty());
 assert_eq!(ja3s.fingerprint.len(), 32);
 assert_eq!(ja3s.ssl_version, 771);
 assert_eq!(ja3s.cipher, 0x1301);
 }

 #[test]
 fn test_ja3s_display() {
 let ja3s = JA3S::generate(771, 0x1301, &[0]);
 let displayed = format!("{}", ja3s);
 assert_eq!(displayed, ja3s.fingerprint);
 }

 #[test]
 fn test_ja3_known_fingerprint() {
 // testanalready know JA3 fingerprint
 // this isansimplify Chrome ClientHello
 let ja3 = JA3::generate(
 771, // TLS 1.2
 &[0xc02b, 0xc02f, 0xc00a],
 &[0, 10, 11],
 &[23, 24],
 &[0],
);

 // Validate JA3 stringformatcorrect
 assert!(ja3.ja3_string.contains("771,"));
 assert!(ja3.ja3_string.contains("49195")); // 0xc02b = 49195
 }
}
