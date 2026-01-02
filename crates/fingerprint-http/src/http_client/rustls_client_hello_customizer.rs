//! rustls ClientHello 定制器（optional）
//!
//! 目front只做一件事：**Based on fingerprint-rust 的 `ClientHelloSpec` adjust"extensionencodingorder"**。
//!
//! explain：
//! - rustls 并不一定willsend spec 里列出的allextension；这里will以 rustls actual `used` 为准，
//!   onlypair交set做重排，并把not覆盖's extensions按 rustls defaultorder追加，ensure仍是anvalid的排列。
//! - spec 里may出现multiple GREASE extension（ in realbrowser中它们通常是不同 GREASE value）。
//!   为避免"重复extensiontype"导致 rustls refuse，我们will把each GREASE 占bit符map成不同 GREASE value。
//!
//! Note: 此Featuresneedsupport ClientHelloCustomizer  rustls fork，standard rustls 不support。
//! currentstandard rustls versionexcluding ClientHelloCustomizer API，therefore此module的代码被暂 when disabled。
//! 如需use此Features，needusesupport ClientHelloCustomizer  rustls fork（如 vistone-rustls）。

// 暂 when disabled整个module，becausestandard rustls 不support ClientHelloCustomizer API
// 如需enabled，needusesupport该 API  rustls fork
// whenusesupport ClientHelloCustomizer  rustls fork  when ，取消down面的comment并enabled相关代码
#![cfg(false)] // 暂 when disabled，becausestandard rustls 不support

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
///  from  `ClientHelloSpec` Calculate"期望's extensionsorder"（以 u16 represent）。
///
/// - deduplicate：同一extensiontype只preserve第一次出现
/// - GREASE：把重复 GREASE 占bit符map成不同 GREASE value
fn desired_extension_ids_from_spec(spec: &ClientHelloSpec) -> Vec<u16> {
    let mut out: Vec<u16> = Vec::with_capacity(spec.extensions.len());
    let mut grease_cursor = 0usize;

    for ext in &spec.extensions {
        let mut id = ext.extension_id();

        // process GREASE：尽量给each GREASE allocate不同的value，以符合"多 GREASE extension"的现实形态。
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
/// will rustls current `used` 's extensionsorder，按 `desired`（来自 spec）重排。
///
/// 规则：
/// - 只pair `used` 里出现's extensions做重排（交set）
/// - `desired` 里重复/不 in `used` 的will被ignore
/// - `used` 里not出现 in `desired` 's extensionskeep原相pairorder并追加 to end尾
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
/// based on `ClientProfile`  ClientHello extensionorder定制器。
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
