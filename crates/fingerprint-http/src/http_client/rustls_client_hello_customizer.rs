//! rustls ClientHello customizeer (optional)
//!
//! 目front只做一件事：**Based on fingerprint-rust `ClientHelloSpec` adjust"extensionencodingorder"**。
//!
//! explain：
//! - rustls 并不一定willsend spec 里listallextension；herewill以 rustls actual `used` as 准，
//! onlypair交setreorder，并把notcover's extensions by  rustls defaultorder追加，ensure仍 is anvalidarrange。
//! - spec 里mayappearmultiple GREASE extension ( in realbrowser in themusually is different GREASE value)。
//! as avoid"duplicateextensiontype"cause rustls refuse，wewill把each GREASE 占bit符map成different GREASE value。
//!
//! Note: 此Featuresneedsupport ClientHelloCustomizer rustls fork，standard rustls 不support。
//! currentstandard rustls versionexcluding ClientHelloCustomizer API，therefore此modulecode被暂 when disabled。
//! if neededuse此Features，needusesupport ClientHelloCustomizer rustls fork (如 vistone-rustls)。

// 暂 when disabled整module，becausestandard rustls 不support ClientHelloCustomizer API
// if neededenabled，needusesupport该 API rustls fork
// whenusesupport ClientHelloCustomizer rustls fork when ，canceldown面comment并enabled相closecode
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
/// from `ClientHelloSpec` Calculate"expected's extensionsorder" (以 u16 represent)。
///
/// - deduplicate：sameextensiontype只preserve第onceappear
/// - GREASE：把duplicate GREASE 占bit符map成different GREASE value
fn desired_extension_ids_from_spec(spec: &ClientHelloSpec) -> Vec<u16> {
 let mut out: Vec<u16> = Vec::with_capacity(spec.extensions.len());
 let mut grease_cursor = 0usize;

 for ext in &spec.extensions {
 let mut id = ext.extension_id();

 // process GREASE：尽量给each GREASE allocatedifferentvalue，以符合"多 GREASE extension"现实形态。
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
/// will rustls current `used` 's extensionsorder， by  `desired` (from spec)reorder。
///
/// rule：
/// - 只pair `used` 里appear's extensionsreorder (交set)
/// - `desired` 里duplicate/不 in `used` will被ignore
/// - `used` 里notappear in `desired` 's extensionskeep原相pairorder并追加 to end尾
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
/// based on `ClientProfile` ClientHello extensionordercustomizeer。
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
