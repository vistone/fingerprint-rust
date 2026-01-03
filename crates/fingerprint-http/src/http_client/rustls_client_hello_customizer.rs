//! rustls ClientHello customizeer (optional)
//!
//! item front only 做一件事：**Based on fingerprint-rust `ClientHelloSpec` adjust"extensionencodingorder"**。
//!
//! explain ：
//! - rustls not 一定 will send spec 里list all extension；here will to rustls actual `used` as 准，
//! onlypair交setreorder， and 把not cover 's extensions by rustls defaultorder追加，ensure仍 is anvalidarrange。
//! - spec 里mayappearmultiple GREASE extension (in realbrowser in themusu all y is different GREASE value)。
//! as avoid "duplicateextensiontype"cause rustls refuse，we will 把each GREASE 占bit符map成 different GREASE value。
//!
//! Note: this Featuresneedsupport ClientHelloCustomizer rustls fork，standard rustls not support。
//! currentstandard rustls versionexcluding ClientHelloCustomizer API，therefore this modulecode by temporary when disabled。
//! if neededuse this Features，needusesupport ClientHelloCustomizer rustls fork (such as vistone-rustls)。

// temporary when disabled整module，becausestandard rustls not support ClientHelloCustomizer API
// if neededenabled，needusesupport该 API rustls fork
// when usesupport ClientHelloCustomizer rustls fork when ，canceldown面comment and enabled phase closecode
#![cfg(false)] // temporary when disabled，becausestandard rustls not support

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
/// from `ClientHelloSpec` Calculate"expected's extensionsorder" (to u16 represent)。
///
/// - deduplicate：sameextensiontype only preserve第onceappear
/// - GREASE：把duplicate GREASE 占bit符map成 different GREASE value
fn desired_extension_ids_from_spec(spec: &ClientHelloSpec) -> Vec<u16> {
 let mut out: Vec<u16> = Vec::with_capacity(spec.extensions.len());
 let mut grease_cursor = 0usize;

 for ext in &spec.extensions {
 let mut id = ext.extension_id();

 // process GREASE：尽量给each GREASE all ocate differentvalue， to 符合" many GREASE extension"现实形态。
 if is_grease_value(id) {
 for _ in 0..TLS_GREASE_VALUES.len() {
 let candidate = TLS_GREASE_VALUES[grease_cursor % TLS_GREASE_VALUES.len()];
 grease_cursor += 1;
 if!out.contains(&candidate) {
 id = candidate;
 break;
 }
 }
 }

 if!out.contains(&id) {
 out.push(id);
 }
 }

 out
}

#[cfg(feature = "rustls-client-hello-customizer")]
/// will rustls current `used` 's extensionsorder， by `desired` (from spec)reorder。
///
/// rule：
/// - only pair `used` 里appear's extensionsreorder (交set)
/// - `desired` 里duplicate/ not in `used` will by ignore 
/// - `used` 里notappear in `desired` 's extensionskeep original phase pairorder and 追加 to end尾
fn reorder_used_extensions(used: Vec<ExtensionType>, desired: &[u16]) -> Vec<ExtensionType> {
 let mut out: Vec<ExtensionType> = Vec::with_capacity(used.len());

 for id in desired {
 let ty = ExtensionType::from(*id);
 if used.contains(&ty) &&!out.contains(&ty) {
 out.push(ty);
 }
 }

 for ty in used {
 if!out.contains(&ty) {
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
 pub fn try _from_profile(profile: &ClientProfile) -> Option<Self> {
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
