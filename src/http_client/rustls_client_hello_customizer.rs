//! rustls ClientHello 定制器（路线 A）
//!
//! 使用 `vistone/rustls` 的 backport（0.21）来获得 ClientHello customizer hook：
//! - **扩展编码顺序重排**：按 `ClientHelloSpec.extensions` 的顺序尽量贴近浏览器
//! - 另外我们也会从 spec 提取 “cipher suites / kx groups / versions” 供 builder 使用（见 `ProfileClientHelloParams`）

use std::sync::Arc;

use crate::dicttls::supported_groups;
use crate::tls_config::{is_grease_value, TLS_GREASE_VALUES, VERSION_TLS12, VERSION_TLS13};
use crate::tls_extensions::{KeyShareExtension, SupportedCurvesExtension, SupportedVersionsExtension, UtlsPaddingExtension};
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
        // RFC8446: pre_shared_key 必须最后；这里先跳过，最后再处理
        if ty == ExtensionType::PreSharedKey {
            continue;
        }
        if used.contains(&ty) && !out.contains(&ty) {
            out.push(ty);
        }
    }

    for ty in used {
        if !out.contains(&ty) {
            out.push(ty);
        }
    }

    if out.contains(&ExtensionType::PreSharedKey) && out.last() != Some(&ExtensionType::PreSharedKey) {
        out.retain(|t| *t != ExtensionType::PreSharedKey);
        out.push(ExtensionType::PreSharedKey);
    }

    out
}

/// 基于 `ClientProfile` 的 ClientHello 扩展顺序定制器。
#[derive(Debug)]
pub struct ProfileClientHelloCustomizer {
    desired_extension_ids: Vec<u16>,
    wants_padding: bool,
    wants_grease_named_group: bool,
    wants_grease_keyshare: bool,
    desired_named_groups: Vec<u16>,
}

impl ProfileClientHelloCustomizer {
    pub fn try_from_profile(profile: &ClientProfile) -> Option<Self> {
        let spec = profile.get_client_hello_spec().ok()?;
        let wants_padding = spec.extensions.iter().any(|e| e.extension_id() == 0x0015);
        let mut wants_grease_named_group = false;
        let mut wants_grease_keyshare = false;
        let mut desired_named_groups: Vec<u16> = Vec::new();
        for ext in &spec.extensions {
            if let Some(sc) = ext.as_any().downcast_ref::<SupportedCurvesExtension>() {
                if sc.curves.iter().any(|g| is_grease_value(*g)) {
                    wants_grease_named_group = true;
                }
                for g in &sc.curves {
                    let mut id = *g;
                    if is_grease_value(id) {
                        id = TLS_GREASE_VALUES[0];
                    }
                    if !desired_named_groups.contains(&id) {
                        desired_named_groups.push(id);
                    }
                }
            } else if let Some(ks) = ext.as_any().downcast_ref::<KeyShareExtension>() {
                if ks.key_shares.iter().any(|k| is_grease_value(k.group)) {
                    wants_grease_keyshare = true;
                }
            }
        }
        Some(Self {
            desired_extension_ids: desired_extension_ids_from_spec(&spec),
            wants_padding,
            wants_grease_named_group,
            wants_grease_keyshare,
            desired_named_groups,
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
        // 0) “不会破坏握手”的内容级微调：
        // - supported_groups: 注入 GREASE group + 按 spec 尽量排序（仅对 rustls 现有组）
        // - key_share: 在现有 key_share 前插入 GREASE entry（避免影响 rustls 的实际 key exchange 状态）
        if self.wants_grease_named_group {
            if let Some(current) = hello.named_groups() {
                // 只保留 rustls 当前支持/已声明的 group（避免 HRR 要求我们不支持的组）
                let mut desired: Vec<u16> = Vec::new();

                for g in &self.desired_named_groups {
                    if is_grease_value(*g) || *g == TLS_GREASE_VALUES[0] {
                        // 允许 GREASE group
                        if !desired.contains(&TLS_GREASE_VALUES[0]) {
                            desired.push(TLS_GREASE_VALUES[0]);
                        }
                        continue;
                    }

                    if current.contains(g) && !desired.contains(g) {
                        desired.push(*g);
                    }
                }

                // 追加 rustls 当前的 groups（保持握手兼容性）
                for g in current.iter().copied() {
                    if !desired.contains(&g) {
                        desired.push(g);
                    }
                }

                // set_named_groups 允许 Unknown；我们已保证至少包含 rustls 当前的 groups
                let _ = hello.set_named_groups(desired);
            }
        }

        if self.wants_grease_keyshare {
            if let Some(mut entries) = hello.key_share_entries() {
                let grease_group = TLS_GREASE_VALUES[0];
                // GREASE keyshare payload：uTLS 常用单字节占位即可
                if !entries.iter().any(|(g, _)| *g == grease_group) {
                    entries.insert(0, (grease_group, vec![0u8]));
                    let _ = hello.set_key_share_entries(entries);
                }
            }
        }

        // 1) 注入 rustls 默认不会发送、但 spec 期望出现的扩展（尽量不影响握手正确性）
        //
        // 目前只注入“安全的/可空载荷”的扩展：
        // - GREASE 扩展（unknown extension types, empty payload）
        // - SessionTicket (request, empty payload)
        // - RenegotiationInfo（payload = [0x00]）
        //
        // 以及按 BoringSSL 规则计算的 Padding（payload = N 个 0）

        // 先移除已有 padding（避免重复影响长度计算/导致 duplicate）
        let _ = hello.remove_extension(ExtensionType::Padding);

        // 根据 desired 列表，决定需要注入哪些“缺失”扩展
        let used_before = hello.extension_encoding_order();
        for id in &self.desired_extension_ids {
            let ty = ExtensionType::from(*id);

            // 只处理 rustls 当前没有的扩展（避免 duplicate）
            if used_before.contains(&ty) {
                continue;
            }

            // GREASE 扩展：unknown extension type + empty payload
            if is_grease_value(*id) {
                // ignore error on duplicates in case used list changed
                let _ = hello.push_raw_extension(ty, Vec::new());
                continue;
            }

            // SessionTicket：空载荷表示 "request"
            if ty == ExtensionType::SessionTicket {
                let _ = hello.push_raw_extension(ty, Vec::new());
                continue;
            }

            // RenegotiationInfo：初次握手通常为 single zero byte
            // (body: renegotiated_connection_length = 0)
            if ty.get_u16() == 0xff01 {
                let _ = hello.push_raw_extension(ty, vec![0x00]);
                continue;
            }

            // GREASE ECH：0xfe0d，通常空载荷（作为 ECH grease 占位）
            if ty.get_u16() == 0xfe0d {
                let _ = hello.push_raw_extension(ty, Vec::new());
                continue;
            }
        }

        // 2) Padding：按 BoringPaddingStyle（uTLS/Chrome 风格）计算
        // unpadded_len 使用当前 ClientHello payload 的编码长度
        if self.wants_padding {
            let unpadded_len = hello.encoded_len();
            let (padding_len, will_pad) = UtlsPaddingExtension::boring_padding_style(unpadded_len);
            if will_pad && padding_len > 0 {
                let _ = hello.push_raw_extension(ExtensionType::Padding, vec![0u8; padding_len]);
            }
        }

        // 3) 最后按 spec 重排扩展顺序（psk 强制最后）
        let used = hello.extension_encoding_order();
        let order = reorder_used_extensions(used, &self.desired_extension_ids);
        hello.set_extension_encoding_order(order)?;
        Ok(())
    }
}
