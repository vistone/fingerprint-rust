# Security Policy

## Supported Versions

We take security seriously and provide security updates for the following versions:

| 版本 (Version) | Supported          | Notes                        |
| ------- | ------------------ | ---------------------------- |
| 2.1.x   | :white_check_mark: | Current stable release       |
| 2.0.x   | :white_check_mark: | LTS, security fixes only     |
| < 2.0   | :x:                | End of life, upgrade advised |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them privately via one of the following methods:

### 1. GitHub Security Advisories (Preferred)

1. Go to https://github.com/vistone/fingerprint-rust/security/advisories
2. Click "New draft security advisory"
3. Provide detailed information about the vulnerability
4. Submit the advisory

### 2. Email

Send details to the project maintainers (contact information in repository)

### What to Include

When reporting a vulnerability, please include:

- **Description**: Clear description of the vulnerability
- **Impact**: What can an attacker achieve?
- **Reproduction**: Step-by-step instructions to reproduce
- **Affected versions**: Which versions are vulnerable?
- **Suggested fix**: If you have one (optional)
- **Proof of concept**: Code, packet captures, etc. (if applicable)

### Example Report

```markdown
**Summary**: Buffer overflow in packet parsing

**Impact**: Remote code execution possible with crafted IPv4 packet

**Reproduction**:
1. Create malformed IPv4 packet with IHL < 5
2. Send to application
3. Application crashes with segfault

**Affected Versions**: 2.0.0 - 2.1.0

**Suggested Fix**: Add validation: if ihl < 5 || ihl > 15 { return error }

**POC**: [attached pcap file]
```

## Response Timeline

- **Initial Response**: Within 48 hours
- **Assessment**: Within 7 days
- **Fix Development**: Depends on severity
  - Critical: 1-7 days
  - High: 7-14 days
  - Medium: 14-30 days
  - Low: 30-90 days
- **Public Disclosure**: After fix is available

## Severity Classification

We use CVSS v3.1 for severity assessment:

| Severity | CVSS Score | Response Time |
|----------|------------|---------------|
| Critical | 9.0-10.0   | 1-7 days      |
| High     | 7.0-8.9    | 7-14 days     |
| Medium   | 4.0-6.9    | 14-30 days    |
| Low      | 0.1-3.9    | 30-90 days    |

## Security Update Process

1. **Develop Fix**: Create patch in private branch
2. **Internal Review**: Security team reviews fix
3. **测试**: Comprehensive 测试 including regression tests
4. **Prepare Release**: Create new 版本 (Version) with fix
5. **Coordinate Disclosure**: 
   - Notify reporter
   - Prepare security advisory
   - Update CHANGELOG
6. **Release**: Publish fixed 版本 (Version)
7. **Public Disclosure**: Publish security advisory
8. **Notify Users**: Announce via GitHub, social media, etc.

## What We Consider a Security Issue

### In Scope

- **Memory safety**: Buffer overflows, use-after-free, etc.
- **Denial of Service**: Resource exhaustion, infinite loops
- **Input validation**: Parsing vulnerabilities, injection attacks
- **Information disclosure**: Unintended data leaks
- **Cryptographic issues**: Weak algorithms, improper usage
- **Authentication/Authorization**: Bypass vulnerabilities
- **Code execution**: RCE, arbitrary code execution

### Out of Scope

- Issues in 依赖关系 (report to upstream)
- Issues requiring physical access
- Social engineering attacks
- Denial of service requiring large resources
- Issues only affecting outdated/unsupported versions
- Theoretical vulnerabilities without practical impact

## Vulnerability Disclosure Policy

We follow **Coordinated Vulnerability Disclosure**:

1. **Private Reporting**: Report vulnerabilities privately
2. **Acknowledgment**: We acknowledge receipt within 48 hours
3. **Investigation**: We investigate and validate the report
4. **Development**: We develop and test a fix
5. **Coordination**: We coordinate disclosure timing with reporter
6. **Public Disclosure**: We publicly disclose after fix is available

### Disclosure Timeline

- **Standard**: 90 days after initial report
- **Extended**: If fix requires more time, we may request extension
- **Immediate**: If vulnerability is being actively exploited

## Security Best Practices for Users

### Installation

```toml
# Always use specific versions
[dependencies]
fingerprint = "2.1.0"  # Not "2.1" or "2"
```

### Updates

```bash
# Regularly check for updates
cargo update

# Check for security advisories
cargo audit
```

### 配置

```rust
// Use secure defaults
let config = HttpClientConfig {
    // Enable TLS verification
    verify_ssl: true,
    
    // Set reasonable timeouts
    timeout: Duration::from_secs(30),
    
    // Limit redirects
    max_redirects: 5,
    
    ..Default::default()
};
```

### Network Security

- Use HTTPS for all connections
- Validate SSL/TLS certificates
- Set appropriate timeouts
- Limit request/response sizes
- Implement rate limiting

### Input Validation

```rust
// Always validate external input
fn process_packet(data: &[u8]) -> Result<(), Error> {
    // Check size
    if data.len() < MIN_SIZE || data.len() > MAX_SIZE {
        return Err(Error::InvalidSize);
    }
    
    // Validate format
    if !is_valid_format(data) {
        return Err(Error::InvalidFormat);
    }
    
    // Process...
    Ok(())
}
```

## Security Features

### Current

- ✅ Memory-safe Rust 实现
- ✅ Input validation on all external data
- ✅ Bounds checking on array accesses
- ✅ Safe integer arithmetic
- ✅ TLS 1.3 支持
- ✅ Certificate validation
- ✅ Timeout 保护
- ✅ Size limits on packets/requests
- ✅ DoS 保护 (rate limiting, resource limits)

### Planned

- ⏳ Fuzzing integration (cargo-fuzz)
- ⏳ Property-based 测试 (proptest)
- ⏳ Memory profiling
- ⏳ Static analysis integration (MIRI)
- ⏳ Automated dependency scanning

## Security Audits

### Recent Audits

| Date       | Auditor        | Scope          | Status   |
|------------|----------------|----------------|----------|
| 2026-01-02 | GitHub Copilot | Full codebase  | Complete |

### Audit Reports

Available in:
- `SECURITY_AUDIT.md` - Detailed audit report
- `SECURITY_IMPROVEMENTS.md` - Improvement tracking

## Bug Bounty Program

**Status**: Not currently available

We appreciate security researchers but do not currently offer a formal bug bounty program. However, we will:

- Acknowledge your contribution in release notes
- Credit you in security advisories (if desired)
- Respond promptly and professionally

## Security Champions

Security is everyone's responsibility. However, we have designated security champions:

- **Primary Contact**: Project maintainers
- **Security Review**: All code changes undergo security review
- **Vulnerability Management**: Coordinated through GitHub Security

## Compliance

This project follows:

- **OWASP Top 10** web application security risks
- **CWE Top 25** most dangerous software weaknesses
- **Rust Security Guidelines**
- **NIST Cybersecurity 框架** (where applicable)

## Resources

### Learning

- [Rust Security Book](https://anssi-fr.github.io/rust-guide/)
- [OWASP Cheat Sheets](https://cheatsheetseries.owasp.org/)
- [CWE Top 25](https://cwe.mitre.org/top25/)

### Tools

- `cargo audit` - Check for known vulnerabilities
- `cargo deny` - Dependency policy enforcement
- `cargo fuzz` - Fuzzing 框架
- `clippy` - Linting and best practices

### Community

- GitHub Discussions - For security questions
- GitHub Issues - For non-security bugs
- GitHub Security Advisories - For vulnerability reports

## Changes to This Policy

This security policy may be updated from time to time. Significant changes will be announced via:

- GitHub release notes
- Security advisories
- Project documentation

**最后更新 (Last Updated)**: 2026-01-06  
**版本 (Version)**: 1.0  
**Next Review**: 2026-04-06
