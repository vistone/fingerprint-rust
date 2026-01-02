//! JA3/JA3S TLS 指纹实现
//!
//! JA3 是 Salesforce 开发的 TLS 客户端指纹识别方法，已成为行业标准。
//! JA3S 是对应的服务器端指纹。
//!
//! ## 参考
//! - 论文: "TLS Fingerprinting with JA3 and JA3S" (Salesforce, 2017)
//! - GitHub: https://github.com/salesforce/ja3

use serde::{Deserialize, Serialize};

/// JA3 TLS 客户端指纹
///
/// 格式: MD5(SSLVersion,Ciphers,Extensions,EllipticCurves,EllipticCurvePointFormats)
///
/// ## 示例
/// ```
/// use fingerprint_core::ja3::JA3;
///
/// let ja3 = JA3::generate(
///     771, // TLS 1.2
///     &[0x1301, 0x1302, 0x1303],
///     &[0, 10, 11, 13],
///     &[23, 24, 25],
///     &[0],
/// );
/// assert!(!ja3.fingerprint.is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JA3 {
    /// SSL/TLS 版本（十进制）
    pub ssl_version: u16,
    
    /// 密码套件列表（逗号分隔的十进制）
    pub ciphers: String,
    
    /// 扩展列表（逗号分隔的十进制）
    pub extensions: String,
    
    /// 椭圆曲线列表（逗号分隔的十进制）
    pub elliptic_curves: String,
    
    /// 椭圆曲线点格式列表（逗号分隔的十进制）
    pub ec_point_formats: String,
    
    /// 完整的 JA3 字符串（用于计算哈希）
    pub ja3_string: String,
    
    /// JA3 指纹（MD5 哈希）
    pub fingerprint: String,
}

impl JA3 {
    /// 生成 JA3 指纹
    ///
    /// # 参数
    /// - `ssl_version`: TLS 版本（例如：771 = TLS 1.2, 772 = TLS 1.3）
    /// - `ciphers`: 密码套件列表（十六进制值）
    /// - `extensions`: 扩展列表（十六进制值）
    /// - `elliptic_curves`: 椭圆曲线列表（十六进制值）
    /// - `ec_point_formats`: 椭圆曲线点格式列表（十六进制值）
    ///
    /// # 返回
    /// JA3 指纹结构
    pub fn generate(
        ssl_version: u16,
        ciphers: &[u16],
        extensions: &[u16],
        elliptic_curves: &[u16],
        ec_point_formats: &[u8],
    ) -> Self {
        // 过滤 GREASE 值（如果需要）
        let filtered_ciphers: Vec<u16> = ciphers
            .iter()
            .filter(|&&c| !crate::grease::is_grease_value(c))
            .cloned()
            .collect();

        let filtered_extensions: Vec<u16> = extensions
            .iter()
            .filter(|&&e| !crate::grease::is_grease_value(e))
            .cloned()
            .collect();

        let filtered_curves: Vec<u16> = elliptic_curves
            .iter()
            .filter(|&&c| !crate::grease::is_grease_value(c))
            .cloned()
            .collect();

        // 转换为逗号分隔的十进制字符串（JA3 使用十进制，不是十六进制）
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

        // 构建 JA3 字符串
        let ja3_string = format!(
            "{},{},{},{},{}",
            ssl_version, ciphers_str, extensions_str, curves_str, formats_str
        );

        // 计算 MD5 哈希
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

    /// 计算 MD5 哈希
    fn md5_hash(input: &str) -> String {
        // MD5 computation
        let digest = md5::compute(input.as_bytes());
        // Computed above
        format!("{:x}", digest)
    }

    /// 从 ClientHello 原始数据生成 JA3
    ///
    /// 这是一个便捷方法，用于从完整的 ClientHello 消息中提取并生成 JA3
    pub fn from_client_hello(client_hello: &crate::signature::ClientHelloSignature) -> Self {
        // 转换椭圆曲线 CurveID 为 u16
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

/// JA3S TLS 服务器指纹
///
/// 格式: MD5(SSLVersion,Cipher,Extensions)
///
/// ## 示例
/// ```
/// use fingerprint_core::ja3::JA3S;
///
/// let ja3s = JA3S::generate(771, 0x1301, &[0, 10, 11]);
/// assert!(!ja3s.fingerprint.is_empty());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JA3S {
    /// SSL/TLS 版本（十进制）
    pub ssl_version: u16,
    
    /// 选择的密码套件（十进制）
    pub cipher: u16,
    
    /// 扩展列表（逗号分隔的十进制）
    pub extensions: String,
    
    /// 完整的 JA3S 字符串（用于计算哈希）
    pub ja3s_string: String,
    
    /// JA3S 指纹（MD5 哈希）
    pub fingerprint: String,
}

impl JA3S {
    /// 生成 JA3S 指纹
    ///
    /// # 参数
    /// - `ssl_version`: TLS 版本
    /// - `cipher`: 服务器选择的密码套件
    /// - `extensions`: 服务器返回的扩展列表
    pub fn generate(ssl_version: u16, cipher: u16, extensions: &[u16]) -> Self {
        // 过滤 GREASE 值
        let filtered_extensions: Vec<u16> = extensions
            .iter()
            .filter(|&&e| !crate::grease::is_grease_value(e))
            .cloned()
            .collect();

        // 转换为逗号分隔的十进制字符串
        let extensions_str = filtered_extensions
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<String>>()
            .join("-");

        // 构建 JA3S 字符串
        let ja3s_string = format!("{},{},{}", ssl_version, cipher, extensions_str);

        // 计算 MD5 哈希
        let fingerprint = Self::md5_hash(&ja3s_string);

        Self {
            ssl_version,
            cipher,
            extensions: extensions_str,
            ja3s_string,
            fingerprint,
        }
    }

    /// 计算 MD5 哈希
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
        // 测试 Chrome 浏览器的典型 ClientHello
        let ja3 = JA3::generate(
            771, // TLS 1.2
            &[0x1301, 0x1302, 0x1303, 0xc02b, 0xc02f],
            &[0, 10, 11, 13, 16, 23],
            &[23, 24, 25],
            &[0],
        );

        assert!(!ja3.fingerprint.is_empty());
        assert_eq!(ja3.fingerprint.len(), 32); // MD5 哈希长度
        assert_eq!(ja3.ssl_version, 771);
    }

    #[test]
    fn test_ja3_with_grease() {
        // 测试包含 GREASE 值的情况
        let ja3 = JA3::generate(
            771,
            &[0x0a0a, 0x1301, 0x1a1a], // 包含 GREASE
            &[0x0a0a, 0, 10],          // 包含 GREASE
            &[0x0a0a, 23],             // 包含 GREASE
            &[0],
        );

        // GREASE 值应该被过滤掉
        assert!(!ja3.ciphers.contains("2570")); // 0x0a0a = 2570
        assert!(!ja3.extensions.contains("2570"));
    }

    #[test]
    fn test_ja3_empty_fields() {
        // 测试空字段
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
        // 测试一个已知的 JA3 指纹
        // 这是一个简化的 Chrome ClientHello
        let ja3 = JA3::generate(
            771, // TLS 1.2
            &[0xc02b, 0xc02f, 0xc00a],
            &[0, 10, 11],
            &[23, 24],
            &[0],
        );

        // 验证 JA3 字符串格式正确
        assert!(ja3.ja3_string.contains("771,"));
        assert!(ja3.ja3_string.contains("49195")); // 0xc02b = 49195
    }
}
