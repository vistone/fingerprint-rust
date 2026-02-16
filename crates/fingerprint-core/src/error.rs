//! Comprehensive error types for fingerprint operations
//!
//! Provides strongly-typed error handling to replace string-based errors
//! throughout the codebase, improving debugging and error handling.

use std::fmt;

/// Main error type for fingerprint operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FingerprintError {
    /// TLS-related errors
    Tls(TlsError),

    /// HTTP-related errors
    Http(HttpError),

    /// TCP-related errors
    Tcp(TcpError),

    /// DNS-related errors
    Dns(DnsError),

    /// Parsing errors
    Parse(ParseError),

    /// I/O errors (converted to string for Clone support)
    Io(String),

    /// Configuration errors
    Config(ConfigError),

    /// Cache errors
    Cache(CacheError),

    /// Database errors
    Database(DatabaseError),

    /// Rate limiting errors
    RateLimit(RateLimitError),

    /// Validation errors
    Validation(ValidationError),

    /// Generic error with message
    Other(String),
}

impl fmt::Display for FingerprintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Tls(e) => write!(f, "TLS error: {}", e),
            Self::Http(e) => write!(f, "HTTP error: {}", e),
            Self::Tcp(e) => write!(f, "TCP error: {}", e),
            Self::Dns(e) => write!(f, "DNS error: {}", e),
            Self::Parse(e) => write!(f, "Parse error: {}", e),
            Self::Io(e) => write!(f, "I/O error: {}", e),
            Self::Config(e) => write!(f, "Configuration error: {}", e),
            Self::Cache(e) => write!(f, "Cache error: {}", e),
            Self::Database(e) => write!(f, "Database error: {}", e),
            Self::RateLimit(e) => write!(f, "Rate limit error: {}", e),
            Self::Validation(e) => write!(f, "Validation error: {}", e),
            Self::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for FingerprintError {}

/// TLS-specific errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TlsError {
    /// Invalid TLS version
    InvalidVersion(String),

    /// Invalid cipher suite
    InvalidCipherSuite(u16),

    /// Invalid extension
    InvalidExtension(u16),

    /// Missing required extension
    MissingExtension(String),

    /// Invalid ClientHello
    InvalidClientHello(String),

    /// Invalid ServerHello
    InvalidServerHello(String),

    /// Handshake failed
    HandshakeFailed(String),

    /// Certificate error
    CertificateError(String),

    /// ALPN negotiation failed
    AlpnNegotiationFailed,

    /// SNI validation failed
    SniValidationFailed(String),
}

impl fmt::Display for TlsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidVersion(v) => write!(f, "Invalid TLS version: {}", v),
            Self::InvalidCipherSuite(cs) => write!(f, "Invalid cipher suite: 0x{:04x}", cs),
            Self::InvalidExtension(ext) => write!(f, "Invalid extension: 0x{:04x}", ext),
            Self::MissingExtension(name) => write!(f, "Missing required extension: {}", name),
            Self::InvalidClientHello(msg) => write!(f, "Invalid ClientHello: {}", msg),
            Self::InvalidServerHello(msg) => write!(f, "Invalid ServerHello: {}", msg),
            Self::HandshakeFailed(msg) => write!(f, "TLS handshake failed: {}", msg),
            Self::CertificateError(msg) => write!(f, "Certificate error: {}", msg),
            Self::AlpnNegotiationFailed => write!(f, "ALPN negotiation failed"),
            Self::SniValidationFailed(sni) => write!(f, "SNI validation failed for: {}", sni),
        }
    }
}

/// HTTP-specific errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpError {
    /// Invalid HTTP version
    InvalidVersion(String),

    /// Invalid HTTP method
    InvalidMethod(String),

    /// Invalid header
    InvalidHeader(String),

    /// Missing required header
    MissingHeader(String),

    /// Request build failed
    RequestBuildFailed(String),

    /// Response parse failed
    ResponseParseFailed(String),

    /// Decompression failed
    DecompressionFailed(String),

    /// Invalid HTTP/2 frame
    InvalidHttp2Frame(String),

    /// HTTP/2 settings error
    Http2SettingsError(String),

    /// HTTP/3 error
    Http3Error(String),

    /// Connection pool error
    PoolError(String),

    /// Timeout
    Timeout,
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidVersion(v) => write!(f, "Invalid HTTP version: {}", v),
            Self::InvalidMethod(m) => write!(f, "Invalid HTTP method: {}", m),
            Self::InvalidHeader(h) => write!(f, "Invalid header: {}", h),
            Self::MissingHeader(h) => write!(f, "Missing required header: {}", h),
            Self::RequestBuildFailed(msg) => write!(f, "Request build failed: {}", msg),
            Self::ResponseParseFailed(msg) => write!(f, "Response parse failed: {}", msg),
            Self::DecompressionFailed(msg) => write!(f, "Decompression failed: {}", msg),
            Self::InvalidHttp2Frame(msg) => write!(f, "Invalid HTTP/2 frame: {}", msg),
            Self::Http2SettingsError(msg) => write!(f, "HTTP/2 settings error: {}", msg),
            Self::Http3Error(msg) => write!(f, "HTTP/3 error: {}", msg),
            Self::PoolError(msg) => write!(f, "Connection pool error: {}", msg),
            Self::Timeout => write!(f, "Request timeout"),
        }
    }
}

/// TCP-specific errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TcpError {
    /// Connection failed
    ConnectionFailed(String),

    /// Invalid TCP option
    InvalidOption(u8),

    /// Invalid TCP flags
    InvalidFlags(u8),

    /// Socket configuration failed
    SocketConfigFailed(String),

    /// Bind failed
    BindFailed(String),

    /// Listen failed
    ListenFailed(String),

    /// Accept failed
    AcceptFailed(String),
}

impl fmt::Display for TcpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConnectionFailed(msg) => write!(f, "TCP connection failed: {}", msg),
            Self::InvalidOption(opt) => write!(f, "Invalid TCP option: {}", opt),
            Self::InvalidFlags(flags) => write!(f, "Invalid TCP flags: 0x{:02x}", flags),
            Self::SocketConfigFailed(msg) => write!(f, "Socket configuration failed: {}", msg),
            Self::BindFailed(addr) => write!(f, "Failed to bind to address: {}", addr),
            Self::ListenFailed(msg) => write!(f, "Failed to listen: {}", msg),
            Self::AcceptFailed(msg) => write!(f, "Failed to accept connection: {}", msg),
        }
    }
}

/// DNS-specific errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DnsError {
    /// Resolution failed
    ResolutionFailed(String),

    /// Invalid hostname
    InvalidHostname(String),

    /// Timeout
    Timeout,

    /// No records found
    NoRecords(String),

    /// Invalid DNS response
    InvalidResponse(String),
}

impl fmt::Display for DnsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ResolutionFailed(host) => write!(f, "DNS resolution failed for: {}", host),
            Self::InvalidHostname(host) => write!(f, "Invalid hostname: {}", host),
            Self::Timeout => write!(f, "DNS query timeout"),
            Self::NoRecords(host) => write!(f, "No DNS records found for: {}", host),
            Self::InvalidResponse(msg) => write!(f, "Invalid DNS response: {}", msg),
        }
    }
}

/// Parsing errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    /// Invalid format
    InvalidFormat(String),

    /// Unexpected end of data
    UnexpectedEof,

    /// Data too short
    DataTooShort { expected: usize, actual: usize },

    /// Invalid encoding
    InvalidEncoding(String),

    /// Invalid integer
    InvalidInteger(String),

    /// Invalid URL
    InvalidUrl(String),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidFormat(msg) => write!(f, "Invalid format: {}", msg),
            Self::UnexpectedEof => write!(f, "Unexpected end of data"),
            Self::DataTooShort { expected, actual } => {
                write!(f, "Data too short: expected {}, got {}", expected, actual)
            }
            Self::InvalidEncoding(enc) => write!(f, "Invalid encoding: {}", enc),
            Self::InvalidInteger(msg) => write!(f, "Invalid integer: {}", msg),
            Self::InvalidUrl(url) => write!(f, "Invalid URL: {}", url),
        }
    }
}

/// Configuration errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    /// Missing required field
    MissingField(String),

    /// Invalid value
    InvalidValue { field: String, value: String },

    /// File not found
    FileNotFound(String),

    /// Parse failed
    ParseFailed(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingField(field) => {
                write!(f, "Missing required configuration field: {}", field)
            }
            Self::InvalidValue { field, value } => {
                write!(f, "Invalid value for field '{}': {}", field, value)
            }
            Self::FileNotFound(path) => write!(f, "Configuration file not found: {}", path),
            Self::ParseFailed(msg) => write!(f, "Configuration parse failed: {}", msg),
        }
    }
}

/// Cache errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CacheError {
    /// Cache miss
    Miss,

    /// Cache full
    Full,

    /// Serialization failed
    SerializationFailed(String),

    /// Deserialization failed
    DeserializationFailed(String),

    /// Redis error
    RedisError(String),

    /// TTL expired
    Expired,
}

impl fmt::Display for CacheError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Miss => write!(f, "Cache miss"),
            Self::Full => write!(f, "Cache is full"),
            Self::SerializationFailed(msg) => write!(f, "Serialization failed: {}", msg),
            Self::DeserializationFailed(msg) => write!(f, "Deserialization failed: {}", msg),
            Self::RedisError(msg) => write!(f, "Redis error: {}", msg),
            Self::Expired => write!(f, "Cache entry expired"),
        }
    }
}

/// Database errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DatabaseError {
    /// Connection failed
    ConnectionFailed(String),

    /// Query failed
    QueryFailed(String),

    /// Not found
    NotFound(String),

    /// Duplicate entry
    DuplicateEntry(String),

    /// Invalid schema
    InvalidSchema(String),
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConnectionFailed(msg) => write!(f, "Database connection failed: {}", msg),
            Self::QueryFailed(msg) => write!(f, "Query failed: {}", msg),
            Self::NotFound(item) => write!(f, "Not found: {}", item),
            Self::DuplicateEntry(item) => write!(f, "Duplicate entry: {}", item),
            Self::InvalidSchema(msg) => write!(f, "Invalid schema: {}", msg),
        }
    }
}

/// Rate limiting errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RateLimitError {
    /// Rate limit exceeded
    Exceeded { limit: u64, window_secs: u64 },

    /// Invalid quota
    InvalidQuota(String),

    /// Storage error
    StorageError(String),
}

impl fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Exceeded { limit, window_secs } => {
                write!(
                    f,
                    "Rate limit exceeded: {} requests per {} seconds",
                    limit, window_secs
                )
            }
            Self::InvalidQuota(msg) => write!(f, "Invalid rate limit quota: {}", msg),
            Self::StorageError(msg) => write!(f, "Rate limit storage error: {}", msg),
        }
    }
}

/// Validation errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// Field validation failed
    FieldValidation { field: String, reason: String },

    /// Range validation failed
    RangeValidation { value: i64, min: i64, max: i64 },

    /// Format validation failed
    FormatValidation { expected: String, actual: String },

    /// Consistency check failed
    ConsistencyCheck(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FieldValidation { field, reason } => {
                write!(f, "Field '{}' validation failed: {}", field, reason)
            }
            Self::RangeValidation { value, min, max } => {
                write!(f, "Value {} out of range [{}, {}]", value, min, max)
            }
            Self::FormatValidation { expected, actual } => {
                write!(
                    f,
                    "Format mismatch: expected '{}', got '{}'",
                    expected, actual
                )
            }
            Self::ConsistencyCheck(msg) => write!(f, "Consistency check failed: {}", msg),
        }
    }
}

/// Result type for fingerprint operations
pub type Result<T> = std::result::Result<T, FingerprintError>;

/// Convenience functions for creating errors
impl FingerprintError {
    /// Create a TLS error
    pub fn tls(error: TlsError) -> Self {
        Self::Tls(error)
    }

    /// Create an HTTP error
    pub fn http(error: HttpError) -> Self {
        Self::Http(error)
    }

    /// Create a TCP error
    pub fn tcp(error: TcpError) -> Self {
        Self::Tcp(error)
    }

    /// Create a DNS error
    pub fn dns(error: DnsError) -> Self {
        Self::Dns(error)
    }

    /// Create a parse error
    pub fn parse(error: ParseError) -> Self {
        Self::Parse(error)
    }

    /// Create a config error
    pub fn config(error: ConfigError) -> Self {
        Self::Config(error)
    }

    /// Create a cache error
    pub fn cache(error: CacheError) -> Self {
        Self::Cache(error)
    }

    /// Create a database error
    pub fn database(error: DatabaseError) -> Self {
        Self::Database(error)
    }

    /// Create a rate limit error
    pub fn rate_limit(error: RateLimitError) -> Self {
        Self::RateLimit(error)
    }

    /// Create a validation error
    pub fn validation(error: ValidationError) -> Self {
        Self::Validation(error)
    }

    /// Create a generic error from a message
    pub fn other<S: Into<String>>(msg: S) -> Self {
        Self::Other(msg.into())
    }
}

// Conversion from std::io::Error
impl From<std::io::Error> for FingerprintError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_error_display() {
        let err = FingerprintError::tls(TlsError::InvalidCipherSuite(0x1301));
        assert!(err.to_string().contains("0x1301"));

        let err = FingerprintError::tls(TlsError::MissingExtension("SNI".to_string()));
        assert!(err.to_string().contains("SNI"));
    }

    #[test]
    fn test_http_error_display() {
        let err = FingerprintError::http(HttpError::InvalidMethod("INVALID".to_string()));
        assert!(err.to_string().contains("INVALID"));

        let err = FingerprintError::http(HttpError::Timeout);
        assert!(err.to_string().contains("timeout"));
    }

    #[test]
    fn test_parse_error_display() {
        let err = FingerprintError::parse(ParseError::DataTooShort {
            expected: 10,
            actual: 5,
        });
        assert!(err.to_string().contains("10"));
        assert!(err.to_string().contains("5"));
    }

    #[test]
    fn test_rate_limit_error() {
        let err = FingerprintError::rate_limit(RateLimitError::Exceeded {
            limit: 100,
            window_secs: 60,
        });
        assert!(err.to_string().contains("100"));
        assert!(err.to_string().contains("60"));
    }

    #[test]
    fn test_validation_error() {
        let err = FingerprintError::validation(ValidationError::RangeValidation {
            value: 150,
            min: 0,
            max: 100,
        });
        assert!(err.to_string().contains("150"));
    }

    #[test]
    fn test_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let fp_err: FingerprintError = io_err.into();
        assert!(matches!(fp_err, FingerprintError::Io(_)));
    }
}
