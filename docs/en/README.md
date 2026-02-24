# Documentation Guide

Welcome to the fingerprint-rust documentation! This directory contains all documentation for the project.

## 📍 Start Here

- **[Documentation Index](INDEX.md)** ← Main documentation hub
- **[Quick Start Guide](guides/QUICKSTART.md)** ← 5-minute setup (⭐ NEW)
- **[Developer Guide](guides/DEVELOPMENT.md)** ← Development environment & contributing (⭐ NEW)
- **[FAQ](FAQ.md)** ← Frequently asked questions

## 📚 Documentation Structure

```
docs/
├── INDEX.md                    # Main documentation hub (START HERE)
├── ARCHITECTURE.md             # System architecture and design
├── CONTRIBUTING.md             # How to contribute to the project
├── SECURITY.md                 # Security policies and guidelines
├── ORGANIZATION.md             # Documentation organization guide
├── CHANGELOG.md                # Version history and release notes
├── API.md                      # API overview
│
├── user-guides/                # User guides and tutorials
│   ├── README.md
│   ├── getting-started.md
│   ├── api-usage.md
│   └── fingerprint-guide.md
│
├── developer-guides/           # Development & troubleshooting
│   ├── README.md
│   ├── FUZZING.md
│   ├── PROFILING.md
│   ├── TROUBLESHOOTING_GUIDE.md
│   ├── TUTORIALS.md
│   ├── contributing.md
│   ├── architecture.md
│   └── TEST_REPORT.md
│
├── guides/                     # Implementation guides
│   ├── README.md
│   ├── QUICKSTART.md          # ⭐ 5-minute quick start
│   ├── DEVELOPMENT.md         # ⭐ Developer guide & setup
│   ├── CAPTURE_BROWSER_FINGERPRINTS.md
│   ├── DNS_INTEGRATION_GUIDE.md
│   ├── HTTP2_INTEGRATION_GUIDE.md
│   ├── OPERATIONS_RUNBOOK.md
│   ├── TCP_FINGERPRINT.md
│   ├── UNIFIED_FINGERPRINT.md
│   └── USAGE_GUIDE.md
│
├── reference/                  # Reference documentation
│   ├── README.md
│   ├── document-management-tools.md
│   ├── technical/              # Technical specifications
│   │   ├── GREASE_NORMALIZATION.md
│   │   ├── HPACK_FINGERPRINTING.md
│   │   ├── PACKET_CAPTURE_IMPLEMENTATION.md
│   │   ├── PSK_0RTT_IMPLEMENTATION.md
│   │   ├── RUSTLS_FINGERPRINT_INTEGRATION.md
│   │   ├── TCP_HANDSHAKE_FINGERPRINTING.md
│   │   ├── TLS_CLIENTHELLO_INTEGRATION_COMPLETE.md
│   │   └── TTL_SCORING_OPTIMIZATION.md
│   └── modules/                # Module documentation (13个)
│
├── architecture/               # Architecture documentation
│   └── [Design documents]
│
├── modules/                    # Module-specific guides (13个)
│   └── [core, tls, http, dns, etc.]
│
├── http-client/                # HTTP client documentation (精简)
│   ├── REMOTE_UPDATE_GUIDE.md (合并)
│   └── REMOTE_UPDATE_QUICK_REFERENCE.md
│
├── security/                   # Security documentation (精简)
│   ├── SECURITY_AUDIT_REPORT.md (合并)
│   └── SECURITY_IMPROVEMENTS.md
│
└── archives/                   # Historical documents & reports
    ├── published-reports/      # Past reports (27个)
    ├── completion-reports/     # Completion documentation
    ├── progress-reports/       # Progress tracking
    ├── phases/                 # Phase-wise documentation
    ├── historical-guides/      # Archived guides
    ├── analysis-reports/       # Analysis reports
    ├── project-docs/           # Project documentation
    └── fingerprint_api_deprecated/  # Deprecated API docs
│       ├── TLS_CLIENTHELLO_INTEGRATION_COMPLETE.md
│       └── [Other specifications]
│
├── architecture/               # Architecture diagrams and docs
├── security/                   # Security-related documentation
├── http-client/                # HTTP client documentation
├── reports/                    # Various reports and analysis
├── project-management/         # Project management documents
├── archives/                   # Historical and archived documents
│   ├── completion-reports/
│   ├── progress-reports/
│   ├── analysis-reports/
│   ├── historical-guides/
│   ├── project-docs/
│   ├── quality-reports/
│   └── fingerprint_api_deprecated/
```

## 🎯 Quick Navigation

### For Different User Types

**👤 Project Users**
- Start with [Quick Start](user-guides/getting-started.md)
- Check [API Usage](user-guides/api-usage.md) for integration

**👨‍💻 Developers**
- Read [Architecture](developer-guides/architecture.md)
- Check [Troubleshooting](developer-guides/TROUBLESHOOTING_GUIDE.md)
- See [Contributing](CONTRIBUTING.md)

**🏢 DevOps/Operations**
- Review [Operations Runbook](guides/OPERATIONS_RUNBOOK.md)
- Check [Security](SECURITY.md)
- See [ORGANIZATION](ORGANIZATION.md)

**🔬 Contributors**
- Read [CONTRIBUTING.md](CONTRIBUTING.md)
- Check [ARCHITECTURE.md](ARCHITECTURE.md)
- Review [Troubleshooting Guide](developer-guides/TROUBLESHOOTING_GUIDE.md)

## ✨ Key Documentation Files

| Document | Purpose | Audience |
|----------|---------|----------|
| INDEX.md | Documentation hub | Everyone |
| ARCHITECTURE.md | System design | Developers, Architects |
| CONTRIBUTING.md | Contribution guidelines | Contributors |
| SECURITY.md | Security policies | Security, Ops |
| CHANGELOG.md | Release notes | Everyone |
| ORGANIZATION.md | Docs structure | Maintainers |

## 📖 Document Status

- ✅ **Core Documentation** - Well-maintained and up-to-date
- ✅ **User Guides** - Complete and current
- ✅ **Developer Guides** - Comprehensive
- ✅ **Technical Specifications** - Detailed and accurate
- 📦 **Archives** - Historical documents for reference

## 🔄 Contributing to Documentation

To contribute or report issues with documentation:

1. Read [CONTRIBUTING.md](CONTRIBUTING.md)
2. Check existing [issues](https://github.com/vistone/fingerprint-rust/issues)
3. Submit improvements via pull request

## 📞 Getting Help

- **General Questions** → Check the [user-guides/](user-guides/) directory
- **Technical Issues** → See [Troubleshooting Guide](developer-guides/TROUBLESHOOTING_GUIDE.md)
- **API Questions** → Review [API Reference](reference/)
- **Bugs/Features** → Open an [issue](https://github.com/vistone/fingerprint-rust/issues)

---

**Last Updated**: 2026-02-14  
**Version**: 2.1.0  
**Maintained By**: fingerprint-rust contributors
