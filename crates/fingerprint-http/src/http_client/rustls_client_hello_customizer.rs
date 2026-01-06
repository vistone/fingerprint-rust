//! rustls ClientHello customizeer (optional)
//!
//! itemfrontonlydoone thing：**Based on fingerprint-rust `ClientHelloSpec` adjust"extensionencodingorder"**.
//!
//! explain：
//! - rustls not necessarilyfixedwillsend spec listallextension；herewillwith rustls actual `used` as allow,
//! onlypairsetreorder, and putnotcover's extensions by rustls defaultorderchaseadd, ensure is anvalidarrange.
//! - spec mayappearmultiple GREASE extension ( in realbrowser in themusually is different GREASE value).
//! as avoid"duplicateextensiontype"cause rustls refuse, wewilleach GREASE placeholdersymbolmapbecomedifferent GREASE value.
//!
//! Note: thisFeaturesneedsupport ClientHelloCustomizer rustls fork, standard rustls notsupport.
//! currentstandard rustls versionexcluding ClientHelloCustomizer API, thereforethismodulecodebetemporary when disabled.
//! if neededusethisFeatures, needusesupport ClientHelloCustomizer rustls fork (such as vistone-rustls). // temporary when disabledwholemodule, becausestandard rustls notsupport ClientHelloCustomizer API
// if neededenabled, needusesupport API rustls fork
// whenusesupport ClientHelloCustomizer rustls fork when , canceldowncommentandenabledrelatedcode
#![cfg(false)] // 暂 when disabled，becausestandard rustls notsupport #[cfg(feature = "rustls-client-hello-customizer")]
use std::sync::Arc; use fingerprint_profiles::profiles::ClientProfile;
#[cfg(feature = "rustls-client-hello-customizer")]
use fingerprint_tls::tls_config::{is_grease_value, ClientHelloSpec, TLS_GREASE_VALUES}; #[cfg(feature = "rustls-client-hello-customizer")]
use rustls::client::{ClientHello, ClientHelloContext, ClientHelloCustomizer};
#[cfg(feature = "rustls-client-hello-customizer")]
use rustls::internal::msgs::enums::ExtensionType; #[cfg(feature = "rustls-client-hello-customizer")]
/// from `ClientHelloSpec` Calculate"expected's extensionsorder" (with u16 represent).
///
/// - deduplicate：sameextensiontypeonlypreserveonceappear
/// - GREASE：duplicate GREASE placeholdersymbolmapbecomedifferent GREASE value
fn desired_extension_ids_from_spec(spec: &ClientHelloSpec) -> Vec<u16> { let mut out: Vec<u16> = Vec::with_capacity(spec.extensions.len()); let mut grease_cursor = 0usize; for ext in &spec.extensions { let mut id = ext.extension_id(); // process GREASE：quantityeach GREASE allocatedifferentvalue, withsymbol"multiple GREASE extension"currentactualstate. if is_grease_value(id) { for _ in 0..TLS_GREASE_VALUES.len() { let candidate = TLS_GREASE_VALUES[grease_cursor % TLS_GREASE_VALUES.len()]; grease_cursor += 1; if !out.contains(&candidate) { id = candidate; break; } } } if !out.contains(&id) { out.push(id); } } out
} #[cfg(feature = "rustls-client-hello-customizer")]
/// will rustls current `used` 's extensionsorder, by `desired` (from spec)reorder.
///
/// rule：
/// - onlypair `used` appear's extensionsreorder (set)
/// - `desired` duplicate/not in `used` willbeignore
/// - `used` notappear in `desired` 's extensionskeeporiginalmutualpairorderandchaseadd to end
fn reorder_used_extensions(used: Vec<ExtensionType>, desired: &[u16]) -> Vec<ExtensionType> { let mut out: Vec<ExtensionType> = Vec::with_capacity(used.len()); for id in desired { let ty = ExtensionType::from(*id); if used.contains(&ty) && !out.contains(&ty) { out.push(ty); } } for ty in used { if !out.contains(&ty) { out.push(ty); } } out
} #[cfg(feature = "rustls-client-hello-customizer")]
/// based on `ClientProfile` ClientHello extensionordercustomizeer.
#[derive(Debug)]
pub struct ProfileClientHelloCustomizer { desired_extension_ids: Vec<u16>,
} #[cfg(feature = "rustls-client-hello-customizer")]
impl ProfileClientHelloCustomizer { pub fn try_from_profile(profile: &ClientProfile) -> Option<Self> { let spec = profile.get_client_hello_spec().ok()?; Some(Self { desired_extension_ids: desired_extension_ids_from_spec(&spec), }) } pub fn into_arc(self) -> Arc<Self> { Arc::new(self) }
} #[cfg(feature = "rustls-client-hello-customizer")]
impl ClientHelloCustomizer for ProfileClientHelloCustomizer { fn customize_client_hello( &self, _ctx: ClientHelloContext<'_>, hello: &mut ClientHello<'_>, ) -> Result<(), rustls::Error> { let used = hello.extension_encoding_order(); let order = reorder_used_extensions(used, &self.desired_extension_ids); hello.set_extension_encoding_order(order)?; Ok(()) }
}
