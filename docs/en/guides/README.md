# Implementation Guides

This directory contains practical implementation guides for fingerprint-rust (v2.0 - Consolidated).

## Available Guides

### Protocol Integration
- **[HTTP/2 Integration Guide](HTTP2_INTEGRATION_GUIDE.md)** - HTTP/2 protocol integration and optimization
- **[DNS Integration Guide](DNS_INTEGRATION_GUIDE.md)** - DNS pre-resolution and caching setup

### Fingerprinting Techniques
- **[Capturing Browser Fingerprints](CAPTURE_BROWSER_FINGERPRINTS.md)** 
  - How to capture and analyze browser fingerprints
  - Firefox-specific techniques (merged from FIREFOX_CAPTURE_GUIDE.md)

- **[TCP Fingerprint Guide](TCP_FINGERPRINT.md)** 
  - TCP-level fingerprint application and synchronization
  - Application examples and best practices (merged from TCP_FINGERPRINT_APPLICATION.md and TCP_FINGERPRINT_SYNC.md)

- **[Unified Fingerprint](UNIFIED_FINGERPRINT.md)** 
  - Unified fingerprinting approach and implementation
  - Code examples and use cases (merged from UNIFIED_FINGERPRINT_EXAMPLE.md)

### Operations & Validation
- **[Usage Guide](USAGE_GUIDE.md)** - General usage guidelines and best practices
- **[Operations Runbook](OPERATIONS_RUNBOOK.md)** - Production operations and troubleshooting

## Consolidation Summary

✅ **Merged** similar guides for better organization:
- CAPTURE_BROWSER_FINGERPRINTS + FIREFOX_CAPTURE_GUIDE
- TCP_FINGERPRINT_APPLICATION + TCP_FINGERPRINT_SYNC  
- UNIFIED_FINGERPRINT + UNIFIED_FINGERPRINT_EXAMPLE

✅ **Removed** duplicates:
- ORGANIZATION_GUIDE.md (use [docs/ORGANIZATION.md](../ORGANIZATION.md) instead)

📁 **From 12 files to 8 files** - More focused documentation

## Historical Guides

Archived phase-specific and experimental guides can be found in [../archives/historical-guides/](../archives/historical-guides/).

---

**Version**: 2.0 (Consolidated)  
**Last Updated**: 2026-02-14  
**Status**: Actively Maintained
