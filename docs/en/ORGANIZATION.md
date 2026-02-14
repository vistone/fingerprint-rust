# Documentation Organization Guide

**Version**: 2.0  
**Date**: 2026-02-13  
**Status**: Completed

## 📋 Overview

This documentation guide explains the organizational structure of the `docs/` directory and the documentation classification rules to ensure that documentation management is orderly and easy to locate.

## 🗂️ Directory Structure

```
docs/
├── README.md              # Documentation Center Homepage
├── INDEX.md               # Complete Documentation Index (Chinese)
├── INDEX.en.md            # Complete Documentation Index (English)
├── ARCHITECTURE.md        # Architecture Overview (Chinese)
├── ARCHITECTURE.en.md     # Architecture Overview (English)
├── API.md                 # API Overview
├── CHANGELOG.md           # Change Log
├── CONTRIBUTING.md        # Contributing Guide
├── SECURITY.md            # Security Information
│
├── architecture/          # Architecture and Design Documentation
│   ├── ARCHITECTURE_EVOLUTION.md
│   ├── BINARY_FORMAT_DESIGN.md
│   ├── HTTP2_SETTINGS_ANALYSIS_DESIGN.md
│   ├── TLS_CLIENTHELLO_PARSING_DESIGN.md
│   └── PHASE_7_3_CLASSIFIER_DESIGN.md
│
├── specifications/        # Technical Specification Documentation
│   ├── GREASE_NORMALIZATION.md
│   ├── HPACK_FINGERPRINTING.md
│   ├── TCP_HANDSHAKE_FINGERPRINTING.md
│   ├── PSK_0RTT_IMPLEMENTATION.md
│   ├── RUSTLS_FINGERPRINT_INTEGRATION.md
│   ├── TLS_CLIENTHELLO_INTEGRATION_COMPLETE.md
│   ├── PACKET_CAPTURE_IMPLEMENTATION.md
│   └── TTL_SCORING_OPTIMIZATION.md
│
├── guides/                # User Guides
│   ├── CAPTURE_BROWSER_FINGERPRINTS.md
│   ├── DNS_INTEGRATION_GUIDE.md
│   ├── TCP_FINGERPRINT_APPLICATION.md
│   ├── TCP_FINGERPRINT_SYNC.md
│   ├── UNIFIED_FINGERPRINT.md
│   ├── UNIFIED_FINGERPRINT_EXAMPLE.md
│   └── USAGE_GUIDE.md
│
├── modules/               # Module Documentation
│   ├── api-noise.md
│   ├── core.md
│   ├── defense.md
│   ├── dns.md
│   ├── headers.md
│   ├── http.md
│   ├── http_client.md
│   ├── ml.md
│   ├── profiles.md
│   ├── tls.md
│   ├── tls_config.md
│   ├── tls_handshake.md
│   └── useragent.md
│
├── developer-guides/      # Developer Guides
│   ├── architecture.md
│   ├── contributing.md
│   ├── FUZZING.md
│   ├── PROFILING.md
│   ├── TEST_REPORT.md
│   ├── TROUBLESHOOTING.md
│   └── TUTORIALS.md
│
├── user-guides/           # User Guides
│   ├── getting-started.md
│   ├── fingerprint-guide.md
│   └── api-usage.md
│
├── http-client/           # HTTP Client Documentation
│   ├── REMOTE_UPDATE_SUMMARY.md
│   ├── REMOTE_UPDATE_INDEX.md
│   ├── REMOTE_UPDATE_QUICK_REFERENCE.md
│   ├── REMOTE_UPDATE_CODE_GUIDE.md
│   └── REMOTE_UPDATE_SOURCE_CODE_OVERVIEW.md
│
├── project-management/    # Project Management Documentation
│   ├── phases/           # Phase Documentation
│   │   ├── archived/     # Historical Phases (Phase 0-8)
│   │   ├── PHASE_1_EXECUTION_REPORT.md
│   │   ├── PHASE_7_4_COMPLETION_REPORT.md
│   │   ├── PHASE_8_DEPLOYMENT_GUIDE.md
│   │   ├── PHASE_8_EXECUTION_SUMMARY.md
│   │   ├── PHASE_8_FINAL_COMPLETION_REPORT.md
│   │   └── PHASE_9_*.md  # Phase 9 Series Documentation
│   ├── reports/          # Execution Reports
│   │   ├── EXECUTION_SUMMARY.md
│   │   ├── PROJECT_ANALYSIS_REPORT.md
│   │   └── SESSION_3_*.md
│   └── unified-phase-9-4.md
│
├── reports/              # Analysis Reports
│   ├── CODE_ALIGNMENT_FINAL_REPORT.md
│   ├── CODE_SYNC_COMPLETION_SUMMARY.md
│   ├── COMPLETE_FILE_MANIFEST.md
│   ├── COMPREHENSIVE_ANALYSIS_AND_FIX_PLAN.md
│   ├── PROJECT_ANALYSIS.md
│   ├── PROJECT_EXECUTION_COMPLETE.md
│   ├── TRANSLATION_STATUS.md
│   └── ...
│
├── security/             # Security Documentation
│   ├── AUDIT_REPORT.md
│   ├── SECURITY_AUDIT.md
│   ├── SECURITY_AUDIT_DETAILED.md
│   └── SECURITY_IMPROVEMENTS.md
│
├── archives/             # Historical Archives
│   ├── analysis-reports/
│   ├── completion-reports/
│   ├── progress-reports/
│   ├── project-docs/
│   └── quality-reports/
│
├── archive/              # Legacy Archives
│   ├── fingerprint_api_deprecated/
│   └── phase9.4/
│
└── reference/            # Reference Documentation
    ├── document-management-tools.md
    ├── guides/
    └── specifications/
```

## 📊 Classification Rules

### 1. Core Documentation (Root Directory)
**Location**: `docs/`  
**Documentation Types**:
- Main index files (INDEX.md)
- Overview documentation (ARCHITECTURE.md, API.md)
- Project metadata (README.md, CHANGELOG.md, CONTRIBUTING.md, SECURITY.md)

**Naming Conventions**:
- Use uppercase letters and underscores
- Support multiple language versions (.en.md, .zh.md)

### 2. Architecture Documentation
**Location**: `docs/architecture/`  
**Documentation Types**:
- System architecture design
- Data structure design
- Architecture evolution records

**Naming Conventions**:
- Descriptive naming, e.g., `BINARY_FORMAT_DESIGN.md`
- Use `_DESIGN` suffix to indicate design documentation

### 3. Technical Specifications
**Location**: `docs/specifications/`  
**Documentation Types**:
- Protocol implementation specifications
- Algorithm implementation specifications
- Technical standard documentation

**Naming Conventions**:
- Technical name + feature description
- e.g., `TCP_HANDSHAKE_FINGERPRINTING.md`

### 4. User Guides
**Location**: `docs/guides/`  
**Documentation Types**:
- Operation guides
- Integration guides
- Best practices

**Naming Conventions**:
- Use `_GUIDE` suffix
- Clear feature names in description

### 5. Module Documentation
**Location**: `docs/modules/`  
**Documentation Types**:
- Detailed documentation for each feature module
- API interface specifications
- Usage examples

**Naming Conventions**:
- Use lowercase letters and hyphens
- Consistent with module names (e.g., `fingerprint-ml` → `ml.md`)

### 6. Developer Documentation
**Location**: `docs/developer-guides/`  
**Documentation Types**:
- Development guides
- Testing documentation
- Debugging documentation

**Naming Conventions**:
- Descriptive feature naming
- May use uppercase (e.g., `FUZZING.md`) or lowercase (e.g., `contributing.md`)

### 7. User Documentation
**Location**: `docs/user-guides/`  
**Documentation Types**:
- Getting started guides
- Usage tutorials
- API usage instructions

**Naming Conventions**:
- Use lowercase letters and hyphens
- Descriptive naming, e.g., `getting-started.md`

### 8. Project Management
**Location**: `docs/project-management/`  
**Documentation Types**:
- Phase planning and reports
- Project execution records
- Roadmaps

**Classification Rules**:
- `phases/` - Phase documentation
- `phases/archived/` - Historical phase archives
- `reports/` - Execution reports

### 9. Report Documentation
**Location**: `docs/reports/`  
**Documentation Types**:
- Analysis reports
- Completion reports
- Status summaries

**Naming Conventions**:
- Use `_REPORT` or `_SUMMARY` suffix
- e.g., `CODE_ALIGNMENT_FINAL_REPORT.md`

### 10. Security Documentation
**Location**: `docs/security/`  
**Documentation Types**:
- Security audits
- Security improvements
- Vulnerability reports

**Naming Conventions**:
- Use `SECURITY_` or `AUDIT_` prefix

### 11. Archived Documentation
**Location**: `docs/archives/` or `docs/archive/`  
**Documentation Types**:
- Historical documentation
- Deprecated feature documentation
- Completed project documentation

**Classification Rules**:
- Organize into subdirectories by documentation type
- Maintain original file names

## 🔄 Organization History

### 2026-02-13 - Comprehensive Organization
**Changes Made**:
1. ✅ Created `architecture/` directory, migrated architecture design documentation
2. ✅ Created `specifications/` directory, migrated technical specification documentation
3. ✅ Organized `guides/` directory, unified user guide documentation
4. ✅ Organized `developer-guides/` directory, migrated development and testing documentation
5. ✅ Archived historical phase reports to `project-management/phases/archived/`
6. ✅ Organized `reports/` directory, migrated various report documentation
7. ✅ Updated `README.md` to reflect the latest structure

## 📝 Maintenance Guide

### When Adding Documentation
1. Determine documentation type and classification
2. Select the appropriate directory
3. Follow naming conventions
4. Update README.md and INDEX.md

### When Deprecating Documentation
1. Move to the corresponding subdirectory in `archives/`
2. Add `[Archived]` marker at the top of the documentation
3. Update index files

### Regular Reviews
- Review documentation structure quarterly
- Clean up outdated documentation
- Update index and classification

## 🎯 Best Practices

### ✅ Recommended Practices
- Use clear documentation naming and accurate descriptions
- Organize by feature and type
- Keep directory hierarchy to no more than 3 levels
- Regularly update index files
- Archive historical documentation rather than deleting

### ❌ Practices to Avoid
- Accumulate large amounts of documentation in the root directory
- Use ambiguous file names
- Create excessively deep directory hierarchies
- Scatter documentation across multiple locations
- Delete historical documentation

## 📞 Contact Information

For questions about documentation organization, please:
- Refer to [README.md](README.md)
- Submit a GitHub Issue
- Contact the project maintainers

---
**Last Updated**: 2026-02-13  
**Maintainers**: fingerprint-rust team
