//! TLS extensionmoduletesting
//! TLS extensions tests.
//! testing各种 TLS extensionof functionality

use fingerprint::is_grease_value;
use fingerprint::tls_extensions::*;

#[test]
fn test_grease_extension() {
    let ext = UtlsGREASEExtension::new();
    // GREASE value should be a valid GREASE value (random each time)
    assert!(
        is_grease_value(ext.extension_id()),
        "Extension ID should be a valid GREASE value"
    );
    assert_eq!(ext.len(), 4);

    let mut buf = vec![0u8; 4];
    let n = ext.read(&mut buf).unwrap();
    assert_eq!(n, 4);
}

#[test]
fn test_sni_extension_empty() {
    let ext = SNIExtension::new(String::new());
    assert_eq!(ext.len(), 0);
    assert!(ext.is_empty());
}

#[test]
fn test_sni_extension() {
    let ext = SNIExtension::new("example.com".to_string());
    assert_eq!(ext.extension_id(), 0);
    assert!(ext.len() > 0);

    let mut buf = vec![0u8; 256];
    let n = ext.read(&mut buf).unwrap();
    assert!(n > 0);
}

#[test]
fn test_status_request_extension() {
    let ext = StatusRequestExtension;
    assert_eq!(ext.extension_id(), 5);
    assert_eq!(ext.len(), 9);

    let mut buf = vec![0u8; 9];
    let n = ext.read(&mut buf).unwrap();
    assert_eq!(n, 9);
}

#[test]
fn test_supported_curves_extension() {
    let curves = vec![0x0017, 0x0018, 0x001d];
    let ext = SupportedCurvesExtension::new(curves);
    assert_eq!(ext.extension_id(), 10);
    assert_eq!(ext.len(), 6 + 2 * 3);

    let mut buf = vec![0u8; 256];
    let n = ext.read(&mut buf).unwrap();
    assert_eq!(n, 12);
}

#[test]
fn test_supported_points_extension() {
    let points = vec![0x00];
    let ext = SupportedPointsExtension::new(points);
    assert_eq!(ext.extension_id(), 11);
    assert_eq!(ext.len(), 5 + 1);

    let mut buf = vec![0u8; 6];
    let n = ext.read(&mut buf).unwrap();
    assert_eq!(n, 6);
}

#[test]
fn test_signature_algorithms_extension() {
    let algs = vec![0x0403, 0x0804, 0x0401];
    let ext = SignatureAlgorithmsExtension::new(algs);
    assert_eq!(ext.extension_id(), 13);
    assert_eq!(ext.len(), 6 + 2 * 3);

    let mut buf = vec![0u8; 256];
    let n = ext.read(&mut buf).unwrap();
    assert_eq!(n, 12);
}

#[test]
fn test_alpn_extension() {
    let protocols = vec!["h2".to_string(), "http/1.1".to_string()];
    let ext = ALPNExtension::new(protocols);
    assert_eq!(ext.extension_id(), 16);
    assert!(ext.len() > 0);

    let mut buf = vec![0u8; 256];
    let n = ext.read(&mut buf).unwrap();
    assert!(n > 0);
}

#[test]
fn test_extended_master_secret_extension() {
    let ext = ExtendedMasterSecretExtension;
    assert_eq!(ext.extension_id(), 23);
    assert_eq!(ext.len(), 4);

    let mut buf = vec![0u8; 4];
    let n = ext.read(&mut buf).unwrap();
    assert_eq!(n, 4);
}

#[test]
fn test_session_ticket_extension() {
    let ext = SessionTicketExtension;
    assert_eq!(ext.extension_id(), 35);
    assert_eq!(ext.len(), 4);

    let mut buf = vec![0u8; 4];
    let n = ext.read(&mut buf).unwrap();
    assert_eq!(n, 4);
}

#[test]
fn test_supported_versions_extension() {
    let versions = vec![0x0304, 0x0303];
    let ext = SupportedVersionsExtension::new(versions);
    assert_eq!(ext.extension_id(), 43);
    assert_eq!(ext.len(), 6 + 2 * 2);

    let mut buf = vec![0u8; 256];
    let n = ext.read(&mut buf).unwrap();
    assert_eq!(n, 10);
}

#[test]
fn test_psk_key_exchange_modes_extension() {
    let modes = vec![0x01];
    let ext = PSKKeyExchangeModesExtension::new(modes);
    assert_eq!(ext.extension_id(), 45);
    assert_eq!(ext.len(), 5 + 1);

    let mut buf = vec![0u8; 6];
    let n = ext.read(&mut buf).unwrap();
    assert_eq!(n, 6);
}

#[test]
fn test_key_share_extension() {
    let key_shares = vec![KeyShare {
        group: 0x001d,
        data: vec![0; 32],
    }];
    let ext = KeyShareExtension::new(key_shares);
    assert_eq!(ext.extension_id(), 51);
    assert!(ext.len() > 0);

    let mut buf = vec![0u8; 256];
    let n = ext.read(&mut buf).unwrap();
    assert!(n > 0);
}

#[test]
fn test_sct_extension() {
    let ext = SCTExtension;
    assert_eq!(ext.extension_id(), 18);
    assert_eq!(ext.len(), 4);

    let mut buf = vec![0u8; 4];
    let n = ext.read(&mut buf).unwrap();
    assert_eq!(n, 4);
}

#[test]
fn test_renegotiation_info_extension() {
    let ext = RenegotiationInfoExtension::new(1);
    assert_eq!(ext.extension_id(), 65281);
    assert_eq!(ext.len(), 5);

    let mut buf = vec![0u8; 5];
    let n = ext.read(&mut buf).unwrap();
    assert_eq!(n, 5);
}

#[test]
fn test_application_settings_extension() {
    let protocols = vec!["h2".to_string()];
    let ext = ApplicationSettingsExtensionNew::new(protocols);
    assert_eq!(ext.extension_id(), 17613);
    assert!(ext.len() > 0);

    let mut buf = vec![0u8; 256];
    let n = ext.read(&mut buf).unwrap();
    assert!(n > 0);
}

#[test]
fn test_compress_cert_extension() {
    let algorithms = vec![0x0002];
    let ext = UtlsCompressCertExtension::new(algorithms);
    assert_eq!(ext.extension_id(), 27);
    assert_eq!(ext.len(), 6 + 2);

    let mut buf = vec![0u8; 8];
    let n = ext.read(&mut buf).unwrap();
    assert_eq!(n, 8);
}

#[test]
fn test_grease_ech_extension() {
    let ext = GREASEEncryptedClientHelloExtension::new();
    assert_eq!(ext.extension_id(), 0xfe0d);
    assert_eq!(ext.len(), 4);

    let mut buf = vec![0u8; 4];
    let n = ext.read(&mut buf).unwrap();
    assert_eq!(n, 4);
}

#[test]
fn test_padding_extension() {
    let mut ext = UtlsPaddingExtension::new();
    assert_eq!(ext.extension_id(), 21);
    assert_eq!(ext.len(), 0);
    assert!(!ext.will_pad);

    // testing BoringPaddingStyle
    let (padding_len, will_pad) = UtlsPaddingExtension::boring_padding_style(256);
    assert!(will_pad);
    assert!(padding_len > 0);

    ext.padding_len = padding_len;
    ext.will_pad = will_pad;
    let mut buf = vec![0u8; 1024];
    let n = ext.read(&mut buf).unwrap();
    assert!(n > 0);
}

#[test]
fn test_extension_from_id() {
    // testing已知ofextension ID
    assert!(extension_from_id(0).is_some()); // SNI
    assert!(extension_from_id(5).is_some()); // Status Request
    assert!(extension_from_id(10).is_some()); // Supported Groups
    assert!(extension_from_id(0x0a0a).is_some()); // GREASE

    // testingunknownofextension ID
    assert!(extension_from_id(9999).is_none());
}

#[test]
fn test_buffer_too_short() {
    let ext = StatusRequestExtension;
    let mut buf = vec![0u8; 2]; // buffer区太小
    let result = ext.read(&mut buf);
    assert!(result.is_err());
}
