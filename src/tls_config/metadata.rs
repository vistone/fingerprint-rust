//! TLS 元数据存储模块
//!
//! 在构建 ClientHelloSpec 时保存扩展的元数据（SNI、ALPN 等）
//! 这样可以在提取签名时获取完整信息

use std::collections::HashMap;

/// TLS 扩展元数据
/// 存储扩展的内部数据，用于后续提取
#[derive(Debug, Clone, Default)]
pub struct ExtensionMetadata {
    /// SNI 值（如果存在）
    pub sni: Option<String>,
    /// ALPN 协议列表（如果存在）
    pub alpn: Option<Vec<String>>,
    /// 椭圆曲线列表（如果存在）
    pub elliptic_curves: Option<Vec<u16>>,
    /// 椭圆曲线点格式（如果存在）
    pub elliptic_curve_point_formats: Option<Vec<u8>>,
    /// 签名算法列表（如果存在）
    pub signature_algorithms: Option<Vec<u16>>,
    /// 支持的版本（如果存在）
    pub supported_versions: Option<Vec<u16>>,
}

/// ClientHelloSpec 的元数据
/// 用于存储扩展的内部数据
#[derive(Debug, Clone, Default)]
pub struct SpecMetadata {
    /// 扩展元数据映射（扩展 ID -> 元数据）
    pub extension_metadata: HashMap<u16, ExtensionMetadata>,
}

impl SpecMetadata {
    /// 创建新的元数据
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置 SNI
    pub fn set_sni(&mut self, sni: String) {
        let metadata = self
            .extension_metadata
            .entry(crate::dicttls::extensions::EXT_TYPE_SERVER_NAME)
            .or_default();
        metadata.sni = Some(sni);
    }

    /// 设置 ALPN
    pub fn set_alpn(&mut self, alpn: Vec<String>) {
        let metadata = self
            .extension_metadata
            .entry(crate::dicttls::extensions::EXT_TYPE_APPLICATION_LAYER_PROTOCOL_NEGOTIATION)
            .or_default();
        metadata.alpn = Some(alpn);
    }

    /// 设置椭圆曲线
    pub fn set_elliptic_curves(&mut self, curves: Vec<u16>) {
        let metadata = self
            .extension_metadata
            .entry(crate::dicttls::extensions::EXT_TYPE_SUPPORTED_GROUPS)
            .or_default();
        metadata.elliptic_curves = Some(curves);
    }

    /// 设置椭圆曲线点格式
    pub fn set_elliptic_curve_point_formats(&mut self, formats: Vec<u8>) {
        let metadata = self
            .extension_metadata
            .entry(crate::dicttls::extensions::EXT_TYPE_EC_POINT_FORMATS)
            .or_default();
        metadata.elliptic_curve_point_formats = Some(formats);
    }

    /// 设置签名算法
    pub fn set_signature_algorithms(&mut self, algorithms: Vec<u16>) {
        let metadata = self
            .extension_metadata
            .entry(crate::dicttls::extensions::EXT_TYPE_SIGNATURE_ALGORITHMS)
            .or_default();
        metadata.signature_algorithms = Some(algorithms);
    }

    /// 设置支持的版本
    pub fn set_supported_versions(&mut self, versions: Vec<u16>) {
        let metadata = self
            .extension_metadata
            .entry(crate::dicttls::extensions::EXT_TYPE_SUPPORTED_VERSIONS)
            .or_default();
        metadata.supported_versions = Some(versions);
    }

    /// 获取 SNI
    pub fn get_sni(&self) -> Option<&String> {
        self.extension_metadata
            .get(&crate::dicttls::extensions::EXT_TYPE_SERVER_NAME)
            .and_then(|m| m.sni.as_ref())
    }

    /// 获取 ALPN
    pub fn get_alpn(&self) -> Option<&Vec<String>> {
        self.extension_metadata
            .get(&crate::dicttls::extensions::EXT_TYPE_APPLICATION_LAYER_PROTOCOL_NEGOTIATION)
            .and_then(|m| m.alpn.as_ref())
    }

    /// 获取第一个 ALPN 协议（用于签名）
    pub fn get_first_alpn(&self) -> Option<String> {
        self.get_alpn().and_then(|alpn| alpn.first().cloned())
    }
}
