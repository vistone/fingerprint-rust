# fingerprint-rust Documentation

Welcome to the fingerprint-rust documentation! Here you'll find everything you need to use and develop with this library.

## 🚀 Getting Started

- **[Quick Start Guide](user-guides/getting-started.md)** - Installation and basic setup
- **[Fingerprint Guide](user-guides/fingerprint-guide.md)** - Browser fingerprint configuration
- **[API Usage Guide](user-guides/api-usage.md)** - REST API usage

## 📚 Core Documentation

### Architecture & Design
- **[System Architecture](ARCHITECTURE.md)** - Complete system design and architecture
- **[Module Design](modules/)** - Detailed module specifications:
  - [Core Module](modules/core.md)
  - [TLS Module](modules/tls.md)
  - [HTTP Module](modules/http.md)
  - [Profiles Module](modules/profiles.md)
  - [Defense Module](modules/defense.md)

### Development
- **[Contributing Guidelines](CONTRIBUTING.md)** - How to contribute to the project
- **[Developer Guides](developer-guides/)** - Development documentation
- **[API Reference](reference/)** - Complete API documentation

### Operations
- **[Security](SECURITY.md)** - Security policies and best practices
- **[Organization](ORGANIZATION.md)** - Documentation organization guide
- **[Changelog](CHANGELOG.md)** - Version history and release notes

## 📦 Module Documentation

Each crate has detailed documentation:
- **fingerprint-core** - Core types and utilities
- **fingerprint-tls** - TLS configuration and handshake
- **fingerprint-http** - HTTP client implementation
- **fingerprint-profiles** - Browser fingerprint profiles
- **fingerprint-defense** - Passive detection and active protection
- **fingerprint-gateway** - API gateway implementation

## 📖 How to Use This Documentation

1. **New Users**: Start with [Quick Start Guide](user-guides/) for basic setup
2. **API Users**: Check [API Reference](reference/) for interface documentation  
3. **Developers**: See [System Architecture](ARCHITECTURE.md) and [Developer Guides](developer-guides/)
4. **Operators**: Review [Security](SECURITY.md) and [Organization](ORGANIZATION.md)

## 🔍 Find What You Need

- **Looking for code examples?** → Check [examples/](../examples/) directory
- **Need performance tips?** → See Developer Guides
- **Want to contribute?** → Read [CONTRIBUTING.md](CONTRIBUTING.md)
- **Have security concerns?** → Review [SECURITY.md](SECURITY.md)

## 📋 Documentation Structure

```
docs/
├── INDEX.md                 # This file - documentation hub
├── ARCHITECTURE.md          # System architecture
├── CONTRIBUTING.md          # Contributing guidelines
├── SECURITY.md             # Security policies
├── ORGANIZATION.md         # Docs organization guide
├── CHANGELOG.md            # Version history
├── user-guides/            # User guides and tutorials
├── developer-guides/       # Development documentation
├── modules/                # Module-specific documentation
├── reference/              # API reference and specs
├── guides/                 # Implementation guides
├── http-client/            # HTTP client documentation
├── security/               # Security audit documentation
└── archives/               # Historical documents & reports
```

## 🎯 Key Resources

- [Implementation Guides](guides/) - Protocol and feature implementation
  - [Browser Fingerprinting](guides/CAPTURE_BROWSER_FINGERPRINTS.md)
  - [TCP Fingerprinting](guides/TCP_FINGERPRINT.md)
  - [Unified Fingerprinting](guides/UNIFIED_FINGERPRINT.md)
  - [DNS Integration](guides/DNS_INTEGRATION_GUIDE.md)
  - [HTTP/2 Integration](guides/HTTP2_INTEGRATION_GUIDE.md)

- [Technical References](reference/technical/)
  - TLS, HTTP/2, TCP, DNS specifications
  - GREASE, HPACK, PSK/0RTT implementation details

- [HTTP Client Documentation](http-client/)
  - Remote update guides and references

- [Security Documentation](security/)
  - Security audit reports and improvements

- [Archived Resources](archives/)
  - All historical reports and phase documentation

### Quick Links by Topic

- **Want to learn about fingerprint features?** → [Fingerprint Usage Guide](user-guides/fingerprint-guide.md)
- **Want to integrate APIs?** → [API Integration Guide](user-guides/api-usage.md)
- **Want to contribute to development?** → [Contributing Guidelines](developer-guides/contributing.md)
- **Having issues or problems?** → [Troubleshooting Guide](user-guides/troubleshooting.md)

### Find by Role

- **New Users** → [Getting Started](user-guides/getting-started.md)
- **Developers** → [Architecture Design](developer-guides/architecture.md)
- **Operations Team** → [Deployment Manual](reference/deployment-manual.md)
- **Project Management** → [Project Roadmap](project-management/roadmap.md)

## 🆘 Getting Help

- **GitHub Issues**: [Submit Issues](https://github.com/vistone/fingerprint-rust/issues)
- **Discussion Forum**: [Community Discussion](https://github.com/vistone/fingerprint-rust/discussions)
- **Mailing List**: project@fingerprint-rust.org
- **Real-time Chat**: [Discord Channel](https://discord.gg/fingerprint-rust)

---
**Last Updated**: 2026-02-13  
**Documentation Version**: v2.1.0