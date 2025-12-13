//! TLS 配置模块
//!
//! 提供真实的 TLS Client Hello 配置，对应 Go 版本的 utls.ClientHelloID
//!
//! ## 使用示例
//!
//! ### 使用预定义的指纹
//! ```rust,no_run
//! use fingerprint::tls_config::ClientHelloSpec;
//! let spec = ClientHelloSpec::chrome_133();
//! ```
//!
//! ### 使用 Builder 模式自定义配置
//! ```rust,no_run
//! use fingerprint::tls_config::ClientHelloSpecBuilder;
//! let (extensions, _metadata) = ClientHelloSpecBuilder::chrome_133_extensions();
//! let spec = ClientHelloSpecBuilder::new()
//!     .cipher_suites(ClientHelloSpecBuilder::chrome_cipher_suites())
//!     .compression_methods(vec![0])
//!     .extensions(extensions)
//!     .build();
//! ```

#[macro_use]
mod macros;
mod builder;
mod comparison;
mod extract;
mod grease;
mod ja4;
mod metadata;
mod observable;
mod signature;
mod spec;
mod stats;
mod version;

pub use builder::ClientHelloSpecBuilder;
pub use comparison::{compare_signatures, compare_specs, find_best_match, FingerprintMatch};
pub use extract::extract_signature;
pub use grease::{filter_grease_values, is_grease_value, remove_grease_values, TLS_GREASE_VALUES};
pub use ja4::{Ja4Fingerprint, Ja4Payload, Ja4RawFingerprint, Ja4Signature, first_last_alpn, hash12};
pub use metadata::{ExtensionMetadata, SpecMetadata};
pub use observable::TlsClientObserved;
pub use signature::ClientHelloSignature;
pub use spec::{
    chrome_103_spec, chrome_133_spec, firefox_133_spec, safari_16_0_spec,
    ClientHelloSpec, CipherSuiteID,
    COMPRESSION_NONE, POINT_FORMAT_UNCOMPRESSED, PSK_MODE_DHE,
    RENEGOTIATE_ONCE_AS_CLIENT, CERT_COMPRESSION_BROTLI,
    VERSION_TLS10, VERSION_TLS11, VERSION_TLS12, VERSION_TLS13,
};
pub use stats::FingerprintStats;
pub use version::TlsVersion;
