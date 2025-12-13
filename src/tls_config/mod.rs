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
//! let spec = ClientHelloSpecBuilder::new()
//!     .cipher_suites(ClientHelloSpecBuilder::chrome_cipher_suites())
//!     .compression_methods(vec![0])
//!     .extensions(ClientHelloSpecBuilder::chrome_133_extensions())
//!     .build();
//! ```

#[macro_use]
mod macros;
mod builder;
mod spec;

pub use builder::ClientHelloSpecBuilder;
pub use spec::{
    chrome_103_spec, chrome_133_spec, firefox_133_spec, safari_16_0_spec,
    ClientHelloSpec, CipherSuiteID,
    COMPRESSION_NONE, POINT_FORMAT_UNCOMPRESSED, PSK_MODE_DHE,
    RENEGOTIATE_ONCE_AS_CLIENT, CERT_COMPRESSION_BROTLI,
    VERSION_TLS10, VERSION_TLS11, VERSION_TLS12, VERSION_TLS13,
};
