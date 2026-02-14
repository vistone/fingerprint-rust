# Reference Documentation

Complete reference documentation for fingerprint-rust.

## 📖 Contents

### Technical Specifications
Technical implementation details and specifications can be found in [technical/](technical/):

- **[GREASE Normalization](technical/GREASE_NORMALIZATION.md)** - TLS GREASE handling
- **[HPACK Fingerprinting](technical/HPACK_FINGERPRINTING.md)** - HTTP/2 header compression fingerprinting
- **[Packet Capture Implementation](technical/PACKET_CAPTURE_IMPLEMENTATION.md)** - Network packet capture and analysis
- **[PSK 0-RTT Implementation](technical/PSK_0RTT_IMPLEMENTATION.md)** - Pre-shared key and 0-RTT resumption
- **[RustLS Fingerprint Integration](technical/RUSTLS_FINGERPRINT_INTEGRATION.md)** - RustLS TLS library integration
- **[TCP Handshake Fingerprinting](technical/TCP_HANDSHAKE_FINGERPRINTING.md)** - TCP-level fingerprint analysis
- **[TLS ClientHello Integration](technical/TLS_CLIENTHELLO_INTEGRATION_COMPLETE.md)** - ClientHello message handling
- **[TTL Scoring Optimization](technical/TTL_SCORING_OPTIMIZATION.md)** - Time-to-live value optimization

### Tool Documentation
- **[Document Management Tools](document-management-tools.md)** - Tools for managing project documentation

## 📚 Using This Reference

- **For protocol implementation details** → Check Technical Specifications folder
- **For API usage** → See [user-guides/](../user-guides/)
- **For architecture details** → See [ARCHITECTURE.md](../ARCHITECTURE.md)
- **For module documentation** → See [modules/](../modules/)

---

**Last Updated**: 2026-02-14  
**Status**: Complete Reference
