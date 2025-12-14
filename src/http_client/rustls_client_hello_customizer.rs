//! rustls ClientHello 定制器（路线 A）
//!
//! 使用 `vistone/rustls` 的 backport（0.21）来获得 ClientHello customizer hook：
//! - **扩展编码顺序重排**：按 `ClientHelloSpec.extensions` 的顺序尽量贴近浏览器
//! - 另外我们也会从 spec 提取 “cipher suites / kx groups / versions” 供 builder 使用（见 `ProfileClientHelloParams`）

use std::sync::Arc;

use crate::dicttls::supported_groups;
use crate::tls_config::{is_grease_value, TLS_GREASE_VALUES, VERSION_TLS12, VERSION_TLS13};
use crate::tls_extensions::{KeyShareExtension, SupportedCurvesExtension, SupportedVersionsExtension};
use crate::{ClientHelloSpec, ClientProfile};

use rustls::client::{ClientHello, ClientHelloContext, ClientHelloCustomizer, ExtensionType};

/// 从 `ClientHelloSpec` 提取“可用于 rustls builder”的参数（cipher suites / kx groups / versions）。
#[derive(Debug, Clone)]
pub struct ProfileClientHelloParams {
    pub cipher_suite_ids: Vec<u16>,
    pub kx_group_ids: Vec<u16>,
    pub versions: Vec<u16>,
}

impl ProfileClientHelloParams {
    pub fn try_from_profile(profile: &ClientProfile) -> Option<Self> {
        let spec = profile.get_client_hello_spec().ok()?;
        Some(Self::from_spec(&spec))
    }

    pub fn from_spec(spec: &ClientHelloSpec) -> Self {
        let cipher_suite_ids = spec
            .cipher_suites
            .iter()
            .copied()
            .filter(|id| !is_grease_value(*id))
            .collect::<Vec<_>>();

        let mut kx_group_ids: Vec<u16> = Vec::new();
        let mut versions: Vec<u16> = Vec::new();

        for ext in &spec.extensions {
            if let Some(sc) = ext.as_any().downcast_ref::<SupportedCurvesExtension>() {
                kx_group_ids.extend(sc.curves.iter().copied().filter(|id| !is_grease_value(*id)));
            } else if let Some(ks) = ext.as_any().downcast_ref::<KeyShareExtension>() {
                for k in &ks.key_shares {
                    if !is_grease_value(k.group) {
                        kx_group_ids.push(k.group);
                    }
                }
            } else if let Some(sv) = ext.as_any().downcast_ref::<SupportedVersionsExtension>() {
                versions.extend(sv.versions.iter().copied().filter(|id| !is_grease_value(*id)));
            }
        }

        fn dedup_keep_order(v: &mut Vec<u16>) {
            let mut out = Vec::with_capacity(v.len());
            for x in v.drain(..) {
                if !out.contains(&x) {
                    out.push(x);
                }
            }
            *v = out;
        }

        dedup_keep_order(&mut kx_group_ids);
        dedup_keep_order(&mut versions);

        if versions.is_empty() {
            if spec.tls_vers_max >= VERSION_TLS13 && spec.tls_vers_min <= VERSION_TLS13 {
                versions.push(VERSION_TLS13);
            }
            if spec.tls_vers_max >= VERSION_TLS12 && spec.tls_vers_min <= VERSION_TLS12 {
                versions.push(VERSION_TLS12);
            }
        }

        if kx_group_ids.is_empty() {
            kx_group_ids = vec![
                supported_groups::X25519,
                supported_groups::CURVE_P256,
                supported_groups::CURVE_P384,
            ];
        }

        Self {
            cipher_suite_ids,
            kx_group_ids,
            versions,
        }
    }
}

/// 从 `ClientHelloSpec` 计算“期望的扩展顺序”（以 u16 表示）。
///
/// - 去重：同一扩展类型只保留第一次出现
/// - GREASE：把重复的 GREASE 占位符映射成不同的 GREASE 值
fn desired_extension_ids_from_spec(spec: &ClientHelloSpec) -> Vec<u16> {
    let mut out: Vec<u16> = Vec::with_capacity(spec.extensions.len());
    let mut grease_cursor = 0usize;

    for ext in &spec.extensions {
        let mut id = ext.extension_id();

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

/// 将 rustls 当前 `used` 的扩展顺序，按 `desired`（来自 spec）重排。
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

/// 基于 `ClientProfile` 的 ClientHello 扩展顺序定制器。
#[derive(Debug)]
pub struct ProfileClientHelloCustomizer {
    desired_extension_ids: Vec<u16>,
}

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
