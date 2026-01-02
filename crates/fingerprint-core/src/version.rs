//! TLS versionmodule
//!
//! provide TLS versionenum and ConvertFeatures
//! reference：Huginn Net TlsVersion design

use std::fmt;

/// TLS versionenum
/// includetraditional SSL version以supportcomplete JA4 specificationcompatible性
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TlsVersion {
 /// TLS 1.3
 V1_3,
 /// TLS 1.2
 V1_2,
 /// TLS 1.1
 V1_1,
 /// TLS 1.0
 V1_0,
 /// SSL 3.0
 Ssl3_0,
 /// SSL 2.0（不support，butpreserveenumvalue）
 Ssl2_0,
 /// not知version
 Unknown(u16),
}

impl TlsVersion {
 /// from u16 valueCreate TlsVersion
 pub fn from_u16(value: u16) -> Self {
 match value {
 0x0304 => TlsVersion::V1_3,
 0x0303 => TlsVersion::V1_2,
 0x0302 => TlsVersion::V1_1,
 0x0301 => TlsVersion::V1_0,
 0x0300 => TlsVersion::Ssl3_0,
 _ => TlsVersion::Unknown(value),
 }
 }

 /// convert to u16 value
 pub fn to_u16(self) -> u16 {
 match self {
 TlsVersion::V1_3 => 0x0304,
 TlsVersion::V1_2 => 0x0303,
 TlsVersion::V1_1 => 0x0302,
 TlsVersion::V1_0 => 0x0301,
 TlsVersion::Ssl3_0 => 0x0300,
 TlsVersion::Ssl2_0 => 0x0200,
 TlsVersion::Unknown(v) => v,
 }
 }

 /// Checkwhether as TLS 1.3
 pub fn is_tls13(self) -> bool {
 matches!(self, TlsVersion::V1_3)
 }

 /// Checkwhether as TLS 1.2 or 更highversion
 pub fn is_tls12_or_higher(self) -> bool {
 matches!(self, TlsVersion::V1_2 | TlsVersion::V1_3)
 }
}

impl fmt::Display for TlsVersion {
 fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
 match self {
 TlsVersion::V1_3 => write!(f, "13"),
 TlsVersion::V1_2 => write!(f, "12"),
 TlsVersion::V1_1 => write!(f, "11"),
 TlsVersion::V1_0 => write!(f, "10"),
 TlsVersion::Ssl3_0 => write!(f, "s3"),
 TlsVersion::Ssl2_0 => write!(f, "s2"),
 TlsVersion::Unknown(_) => write!(f, "00"),
 }
 }
}

impl From<u16> for TlsVersion {
 fn from(value: u16) -> Self {
 Self::from_u16(value)
 }
}

impl From<TlsVersion> for u16 {
 fn from(version: TlsVersion) -> Self {
 version.to_u16()
 }
}

#[cfg(test)]
mod tests {
 use super::*;

 #[test]
 fn test_from_u16() {
 assert_eq!(TlsVersion::from_u16(0x0304), TlsVersion::V1_3);
 assert_eq!(TlsVersion::from_u16(0x0303), TlsVersion::V1_2);
 assert_eq!(TlsVersion::from_u16(0x0301), TlsVersion::V1_0);
 }

 #[test]
 fn test_to_u16() {
 assert_eq!(TlsVersion::V1_3.to_u16(), 0x0304);
 assert_eq!(TlsVersion::V1_2.to_u16(), 0x0303);
 assert_eq!(TlsVersion::V1_0.to_u16(), 0x0301);
 }

 #[test]
 fn test_display() {
 assert_eq!(format!("{}", TlsVersion::V1_3), "13");
 assert_eq!(format!("{}", TlsVersion::V1_2), "12");
 assert_eq!(format!("{}", TlsVersion::V1_0), "10");
 }

 #[test]
 fn test_is_tls13() {
 assert!(TlsVersion::V1_3.is_tls13());
 assert!(!TlsVersion::V1_2.is_tls13());
 }

 #[test]
 fn test_is_tls12_or_higher() {
 assert!(TlsVersion::V1_3.is_tls12_or_higher());
 assert!(TlsVersion::V1_2.is_tls12_or_higher());
 assert!(!TlsVersion::V1_1.is_tls12_or_higher());
 }
}
