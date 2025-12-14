//! rustls ClientHello 定制（可选，走“路线 A”：复用 rustls，仅尽量影响 ClientHello）
//!
//! 背景：上游 crates.io 的 rustls 并不提供“任意重排扩展编码顺序/自定义 GREASE/Padding”等完整 uTLS 能力。
//! 因此本模块在 **不依赖 rustls fork** 的前提下，做“best-effort”的 ClientHello 指纹贴近：
//! - 按 `ClientHelloSpec` 设置 **cipher suites 顺序**（rustls 支持的子集）
//! - 按 spec 设置 **KX groups（曲线）顺序**（rustls 支持的子集）
//! - 按 spec 设置 **TLS 版本集合**（TLS1.2/TLS1.3）
//!
//! 说明：
//! - 这并不能 1:1 复刻 uTLS（缺少扩展顺序/内容级别的完全控制），但这是“路线 A”可稳定落地的第一步。

use crate::dicttls::supported_groups;
use crate::tls_config::{is_grease_value, VERSION_TLS12, VERSION_TLS13};
use crate::tls_extensions::{KeyShareExtension, SupportedCurvesExtension, SupportedVersionsExtension};
use crate::{ClientHelloSpec, ClientProfile};

/// 从 `ClientProfile` 提取“可用于 rustls builder 的指纹参数”。
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
                // KeyShare 也能反映 group 顺序；若 curves 没给（或给的不全），作为补充。
                for k in &ks.key_shares {
                    if !is_grease_value(k.group) {
                        kx_group_ids.push(k.group);
                    }
                }
            } else if let Some(sv) = ext.as_any().downcast_ref::<SupportedVersionsExtension>() {
                versions.extend(sv.versions.iter().copied().filter(|id| !is_grease_value(*id)));
            }
        }

        // 去重但保持顺序
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

        // 如果 spec 没给 supported_versions，用 min/max 兜底
        if versions.is_empty() {
            if spec.tls_vers_max >= VERSION_TLS13 && spec.tls_vers_min <= VERSION_TLS13 {
                versions.push(VERSION_TLS13);
            }
            if spec.tls_vers_max >= VERSION_TLS12 && spec.tls_vers_min <= VERSION_TLS12 {
                versions.push(VERSION_TLS12);
            }
        }

        // 如果 spec 没给 curves/keyshare，用 rustls 常见顺序兜底（按我们库里常量）
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
