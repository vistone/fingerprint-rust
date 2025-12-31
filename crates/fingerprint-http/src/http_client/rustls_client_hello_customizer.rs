//! rustls ClientHello 定制器（可选）
//!
//! 目前只做一件事：**根据 fingerprint-rust 的 `ClientHelloSpec` 调整"扩展编码顺序"**。
//!
//! 说明：
//! - rustls 并不一定会发送 spec 里列出的所有扩展；这里会以 rustls 实际 `used` 为准，
//!   仅对交集做重排，并把未覆盖的扩展按 rustls 默认顺序追加，确保仍是一个有效的排列。
//! - spec 里可能出现多个 GREASE 扩展（在真实浏览器中它们通常是不同的 GREASE 值）。
//!   为避免"重复扩展类型"导致 rustls 拒绝，我们会把每个 GREASE 占位符映射成不同的 GREASE 值。
//!
//! 注意：此功能需要支持 ClientHelloCustomizer 的 rustls fork，标准 rustls 不支持。
//! 当前标准 rustls 版本不包含 ClientHelloCustomizer API，因此此模块的代码被暂时禁用。
//! 如需使用此功能，需要使用支持 ClientHelloCustomizer 的 rustls fork（如 vistone-rustls）。

// 暂时禁用整个模块，因为标准 rustls 不支持 ClientHelloCustomizer API
// 如需启用，需要使用支持该 API 的 rustls fork
// 当使用支持 ClientHelloCustomizer 的 rustls fork 时，取消下面的注释并启用相关代码
#![cfg(false)] // 暂时禁用，因为标准 rustls 不支持

#[cfg(feature = "rustls-client-hello-customizer")]
use std::sync::Arc;

use fingerprint_profiles::profiles::ClientProfile;
#[cfg(feature = "rustls-client-hello-customizer")]
use fingerprint_tls::tls_config::{is_grease_value, ClientHelloSpec, TLS_GREASE_VALUES};

#[cfg(feature = "rustls-client-hello-customizer")]
use rustls::client::{ClientHello, ClientHelloContext, ClientHelloCustomizer};
#[cfg(feature = "rustls-client-hello-customizer")]
use rustls::internal::msgs::enums::ExtensionType;

#[cfg(feature = "rustls-client-hello-customizer")]
/// 从 `ClientHelloSpec` 计算"期望的扩展顺序"（以 u16 表示）。
///
/// - 去重：同一扩展类型只保留第一次出现
/// - GREASE：把重复的 GREASE 占位符映射成不同的 GREASE 值
fn desired_extension_ids_from_spec(spec: &ClientHelloSpec) -> Vec<u16> {
    let mut out: Vec<u16> = Vec::with_capacity(spec.extensions.len());
    let mut grease_cursor = 0usize;

    for ext in &spec.extensions {
        let mut id = ext.extension_id();

        // 处理 GREASE：尽量给每个 GREASE 分配不同的值，以符合"多 GREASE 扩展"的现实形态。
        if is_grease_value(id) {
            for _ in 0..TLS_GREASE_VALUES.len() {
                let candidate = TLS_GREASE_VALUES[grease_cursor % TLS_GREASE_VALUES.len()];
                grease_cursor += 1;
                if !out.contains(&candidate) {
                    id = candidate;
                    break;
                }
            }
        }

        if !out.contains(&id) {
            out.push(id);
        }
    }

    out
}

#[cfg(feature = "rustls-client-hello-customizer")]
/// 将 rustls 当前 `used` 的扩展顺序，按 `desired`（来自 spec）重排。
///
/// 规则：
/// - 只对 `used` 里出现的扩展做重排（交集）
/// - `desired` 里重复/不在 `used` 的会被忽略
/// - `used` 里未出现在 `desired` 的扩展保持原相对顺序并追加到末尾
fn reorder_used_extensions(used: Vec<ExtensionType>, desired: &[u16]) -> Vec<ExtensionType> {
    let mut out: Vec<ExtensionType> = Vec::with_capacity(used.len());

    for id in desired {
        let ty = ExtensionType::from(*id);
        if used.contains(&ty) && !out.contains(&ty) {
            out.push(ty);
        }
    }

    for ty in used {
        if !out.contains(&ty) {
            out.push(ty);
        }
    }

    out
}

#[cfg(feature = "rustls-client-hello-customizer")]
/// 基于 `ClientProfile` 的 ClientHello 扩展顺序定制器。
#[derive(Debug)]
pub struct ProfileClientHelloCustomizer {
    desired_extension_ids: Vec<u16>,
}

#[cfg(feature = "rustls-client-hello-customizer")]
impl ProfileClientHelloCustomizer {
    pub fn try_from_profile(profile: &ClientProfile) -> Option<Self> {
        let spec = profile.get_client_hello_spec().ok()?;
        Some(Self {
            desired_extension_ids: desired_extension_ids_from_spec(&spec),
        })
    }

    pub fn into_arc(self) -> Arc<Self> {
        Arc::new(self)
    }
}

#[cfg(feature = "rustls-client-hello-customizer")]
impl ClientHelloCustomizer for ProfileClientHelloCustomizer {
    fn customize_client_hello(
        &self,
        _ctx: ClientHelloContext<'_>,
        hello: &mut ClientHello<'_>,
    ) -> Result<(), rustls::Error> {
        let used = hello.extension_encoding_order();
        let order = reorder_used_extensions(used, &self.desired_extension_ids);
        hello.set_extension_encoding_order(order)?;
        Ok(())
    }
}
