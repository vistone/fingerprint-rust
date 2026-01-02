//! JA4 fingerprintGeneratemodule
//!
//! implementcomplete JA4 TLS clientfingerprintGenerate
//! 参考：Huginn Net  JA4 implement and 官方 FoxIO 规范

use crate::tls_config::grease::filter_grease_values;
use crate::tls_config::version::TlsVersion;
use sha2::{Digest, Sha256};
use std::fmt;

/// JA4 fingerprint（排序/not排序）
#[derive(Debug, Clone, PartialEq)]
pub enum Ja4Fingerprint {
    /// 排序version（ja4）
    Sorted(String),
    /// not排序version（ja4_o，原beginning顺序）
    Unsorted(String),
}

impl fmt::Display for Ja4Fingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ja4Fingerprint::Sorted(s) => write!(f, "{s}"),
            Ja4Fingerprint::Unsorted(s) => write!(f, "{s}"),
        }
    }
}

impl Ja4Fingerprint {
    /// Get变体name（"ja4"  or  "ja4_o"）
    pub fn variant_name(&self) -> &'static str {
        match self {
            Ja4Fingerprint::Sorted(_) => "ja4",
            Ja4Fingerprint::Unsorted(_) => "ja4_o",
        }
    }

    /// Getfingerprintvalue
    pub fn value(&self) -> &str {
        match self {
            Ja4Fingerprint::Sorted(s) => s,
            Ja4Fingerprint::Unsorted(s) => s,
        }
    }
}

/// JA4 原beginningfingerprint（completeversion，排序/not排序）
#[derive(Debug, Clone, PartialEq)]
pub enum Ja4RawFingerprint {
    /// 排序version（ja4_r）
    Sorted(String),
    /// not排序version（ja4_ro，原beginning顺序）
    Unsorted(String),
}

impl fmt::Display for Ja4RawFingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ja4RawFingerprint::Sorted(s) => write!(f, "{s}"),
            Ja4RawFingerprint::Unsorted(s) => write!(f, "{s}"),
        }
    }
}

impl Ja4RawFingerprint {
    /// Get变体name（"ja4_r"  or  "ja4_ro"）
    pub fn variant_name(&self) -> &'static str {
        match self {
            Ja4RawFingerprint::Sorted(_) => "ja4_r",
            Ja4RawFingerprint::Unsorted(_) => "ja4_ro",
        }
    }

    /// Getfingerprintvalue
    pub fn value(&self) -> &str {
        match self {
            Ja4RawFingerprint::Sorted(s) => s,
            Ja4RawFingerprint::Unsorted(s) => s,
        }
    }
}

/// JA4 载荷struct
/// 遵循官方 FoxIO 规范
#[derive(Debug, Clone, PartialEq)]
pub struct Ja4Payload {
    /// JA4_a: TLS version + SNI + cipher suitecount + extensioncount + ALPN
    pub ja4_a: String,
    /// JA4_b: cipher suite（原beginningstring）
    pub ja4_b: String,
    /// JA4_c: extension + signaturealgorithm（原beginningstring）
    pub ja4_c: String,
    /// JA4 fingerprint（hash，排序/not排序）
    pub full: Ja4Fingerprint,
    /// JA4 原beginningfingerprint（complete，排序/not排序）
    pub raw: Ja4RawFingerprint,
}

///  from  ALPN stringExtractfirst and last字符
/// 非 ASCII 字符替换为 '9'
pub fn first_last_alpn(s: &str) -> (char, char) {
    let replace_nonascii_with_9 = |c: char| {
        if c.is_ascii() {
            c
        } else {
            '9'
        }
    };
    let mut chars = s.chars();
    let first = chars.next().map(replace_nonascii_with_9).unwrap_or('0');
    let last = chars
        .next_back()
        .map(replace_nonascii_with_9)
        .unwrap_or('0');
    (first, if s.len() == 1 { '0' } else { last })
}

/// Generate 12 字符hash（SHA256 的front 12字符）
///
/// SHA256 hash总是产生 64十六进制字符，sofront 12字符总是 exists。
/// 此function for  JA4 fingerprintGenerate。
pub fn hash12(input: &str) -> String {
    let hash = Sha256::digest(input.as_bytes());
    let hash_hex = format!("{:x}", hash);
    // SHA256 hash总是 64十六进制字符，sofront 12字符总是 exists
    // use get() methodsecurity地Get切片，避免潜 in  panic
    hash_hex.get(..12).unwrap_or(&hash_hex).to_string()
}

/// TLS ClientHello signature（ for  JA4 Generate）
#[derive(Debug, Clone)]
pub struct Ja4Signature {
    /// TLS version
    pub version: TlsVersion,
    /// cipher suitelist (including GREASE)
    pub cipher_suites: Vec<u16>,
    /// extensionlist (including GREASE)
    pub extensions: Vec<u16>,
    /// signaturealgorithmlist (including GREASE)
    pub signature_algorithms: Vec<u16>,
    /// Server Name Indication
    pub sni: Option<String>,
    /// Application-Layer Protocol Negotiation
    pub alpn: Option<String>,
}

impl Ja4Signature {
    /// Generate JA4 fingerprint（排序version）
    pub fn generate_ja4(&self) -> Ja4Payload {
        self.generate_ja4_with_order(false)
    }

    /// Generate JA4 fingerprint（原beginning顺序version）
    pub fn generate_ja4_original(&self) -> Ja4Payload {
        self.generate_ja4_with_order(true)
    }

    /// Generate JA4 fingerprint（specified顺序）
    /// original_order: true 表示not排序（原beginning顺序），false 表示排序
    fn generate_ja4_with_order(&self, original_order: bool) -> Ja4Payload {
        // 过滤 GREASE value
        let filtered_ciphers = filter_grease_values(&self.cipher_suites);
        let filtered_extensions = filter_grease_values(&self.extensions);
        let filtered_sig_algs = filter_grease_values(&self.signature_algorithms);

        // protocol标记（TLS 为 't'，QUIC 为 'q'）
        let protocol = "t";

        // TLS version
        let tls_version_str = format!("{}", self.version);

        // SNI 指示器：'d'  if  exists SNI，'i'  if 不 exists
        let sni_indicator = if self.sni.is_some() { "d" } else { "i" };

        // cipher suitecount（2-bit十进制，maximum 99）- use原beginningcount（过滤front）
        let cipher_count = format!("{:02}", self.cipher_suites.len().min(99));

        // extensioncount（2-bit十进制，maximum 99）- use原beginningcount（过滤front）
        let extension_count = format!("{:02}", self.extensions.len().min(99));

        // ALPN first and last字符
        let (alpn_first, alpn_last) = match &self.alpn {
            Some(alpn) => first_last_alpn(alpn),
            None => ('0', '0'),
        };

        // JA4_a format：protocol + version + sni + cipher_count + extension_count + alpn_first + alpn_last
        let ja4_a = format!(
            "{protocol}{tls_version_str}{sni_indicator}{cipher_count}{extension_count}{alpn_first}{alpn_last}"
        );

        // JA4_b: cipher suite（排序 or 原beginning顺序，逗号分隔，4-bit十六进制）- 过滤 GREASE
        let mut ciphers_for_b = filtered_ciphers;
        if !original_order {
            ciphers_for_b.sort_unstable();
        }
        let ja4_b_raw = ciphers_for_b
            .iter()
            .map(|c| format!("{c:04x}"))
            .collect::<Vec<String>>()
            .join(",");

        // JA4_c: extension（排序 or 原beginning顺序，逗号分隔，4-bit十六进制）+ "_" + signaturealgorithm
        let mut extensions_for_c = filtered_extensions;

        //  for 排序version：remove SNI (0x0000)  and ALPN (0x0010) 并排序
        //  for 原beginningversion：保留 SNI/ALPN 并保持原beginning顺序
        if !original_order {
            extensions_for_c.retain(|ext| *ext != 0x0000 && *ext != 0x0010);
            extensions_for_c.sort_unstable();
        }

        let extensions_str = extensions_for_c
            .iter()
            .map(|e| format!("{e:04x}"))
            .collect::<Vec<String>>()
            .join(",");

        // signaturealgorithm不排序（Based on规范），but过滤 GREASE
        let sig_algs_str = filtered_sig_algs
            .iter()
            .map(|s| format!("{s:04x}"))
            .collect::<Vec<String>>()
            .join(",");

        // Based on规范， if 没有signaturealgorithm，string不below划线结尾
        let ja4_c_raw = if sig_algs_str.is_empty() {
            extensions_str
        } else if extensions_str.is_empty() {
            sig_algs_str
        } else {
            format!("{extensions_str}_{sig_algs_str}")
        };

        // Generate JA4_b  and JA4_c 的hash（SHA256 的front 12字符）
        let ja4_b_hash = hash12(&ja4_b_raw);
        let ja4_c_hash = hash12(&ja4_c_raw);

        // JA4 hash：ja4_a + "_" + ja4_b_hash + "_" + ja4_c_hash
        let ja4_hashed = format!("{ja4_a}_{ja4_b_hash}_{ja4_c_hash}");

        // JA4 原beginning：ja4_a + "_" + ja4_b_raw + "_" + ja4_c_raw
        let ja4_raw_full = format!("{ja4_a}_{ja4_b_raw}_{ja4_c_raw}");

        // Based on顺序Create相应的enum变体
        let ja4_fingerprint = if original_order {
            Ja4Fingerprint::Unsorted(ja4_hashed)
        } else {
            Ja4Fingerprint::Sorted(ja4_hashed)
        };

        let ja4_raw_fingerprint = if original_order {
            Ja4RawFingerprint::Unsorted(ja4_raw_full)
        } else {
            Ja4RawFingerprint::Sorted(ja4_raw_full)
        };

        Ja4Payload {
            ja4_a,
            ja4_b: ja4_b_raw,
            ja4_c: ja4_c_raw,
            full: ja4_fingerprint,
            raw: ja4_raw_fingerprint,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_first_last_alpn() {
        assert_eq!(first_last_alpn("h2"), ('h', '2'));
        assert_eq!(first_last_alpn("http/1.1"), ('h', '1'));
        assert_eq!(first_last_alpn("h"), ('h', '0'));
    }

    #[test]
    fn test_hash12() {
        let hash = hash12("test");
        assert_eq!(hash.len(), 12);
    }

    #[test]
    fn test_generate_ja4() {
        let sig = Ja4Signature {
            version: TlsVersion::V1_3,
            cipher_suites: vec![0x0a0a, 0x1301, 0x1302], // including GREASE
            extensions: vec![0x0000, 0x0010, 0x002b],
            signature_algorithms: vec![0x0403, 0x0804],
            sni: Some("example.com".to_string()),
            alpn: Some("h2".to_string()),
        };

        let ja4 = sig.generate_ja4();
        assert!(!ja4.ja4_a.is_empty());
        assert!(!ja4.ja4_b.is_empty());
        assert!(!ja4.ja4_c.is_empty());
        assert_eq!(ja4.full.variant_name(), "ja4");
        assert_eq!(ja4.raw.variant_name(), "ja4_r");
    }

    #[test]
    fn test_generate_ja4_original() {
        let sig = Ja4Signature {
            version: TlsVersion::V1_2,
            cipher_suites: vec![0x0a0a, 0x0017],
            extensions: vec![0x0000],
            signature_algorithms: vec![],
            sni: None,
            alpn: None,
        };

        let ja4 = sig.generate_ja4_original();
        assert_eq!(ja4.full.variant_name(), "ja4_o");
        assert_eq!(ja4.raw.variant_name(), "ja4_ro");
    }
}
