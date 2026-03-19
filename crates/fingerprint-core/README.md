# fingerprint-core

`fingerprint-core` contains the shared low-level building blocks of the workspace. It is not a single high-level fingerprint engine. Instead, it exposes reusable types, parsers, fingerprint formats, utility functions, cache helpers, metrics, and packet-analysis primitives.

## What Lives Here

- browser and operating system types
- fingerprint traits and metadata
- JA3, JA4, JA4H, JA4L, JA4S, JARM, HASSH, and related helpers
- HTTP/2 frame parsing and HPACK analysis
- TCP fingerprinting and packet capture utilities
- cache, metrics, and optional Redis-backed helpers
- rate limiting primitives shared by service layers

## Quick Start

```toml
[dependencies]
fingerprint-core = "2.1"
```

```rust
use fingerprint_core::{infer_browser_from_profile_name, is_mobile_profile, BrowserType, OperatingSystem};

assert_eq!(BrowserType::from_str("chrome"), Some(BrowserType::Chrome));
assert_eq!(OperatingSystem::Windows10.as_str(), "Windows NT 10.0; Win64; x64");

let (browser, version) = infer_browser_from_profile_name("chrome_133");
assert_eq!(browser, "chrome");
assert_eq!(version, "133");
assert!(!is_mobile_profile("chrome_133"));
```

## Main Public Areas

| Area | Representative exports |
| --- | --- |
| Core types | `BrowserType`, `OperatingSystem`, `UserAgentTemplate` |
| Fingerprint traits | `Fingerprint`, `FingerprintComparator`, `FingerprintMetadata` |
| TLS fingerprints | `JA3`, `JA3S`, `JA4`, `JA4H`, `JA4L`, `JA4S`, `JA4T`, `JA4TS`, `JA4X`, `Jarm` |
| SSH fingerprints | `HASSH`, `HASSHServer`, `JA4SSH` |
| HTTP analysis | `HttpFingerprint`, `HpackAnalyzer`, `Http2SettingsFrame`, `Http2SettingsMatcher` |
| Packet/TCP analysis | `PacketParser`, `TcpFingerprint`, `TcpHandshakeAnalyzer` |
| Cache and metrics | `Cache`, `CacheStats`, `PrometheusMetrics` |
| Rate limiting | `RateLimiter`, `QuotaTier`, `RateLimitResponse` |

## Features

Recommended Cargo features for this crate follow a layered model:

```toml
[features]
default = []
service-runtime = ["service-cache", "service-rate-limiting"]
service-observability = ["service-runtime", "service-metrics"]
service-distributed = ["service-runtime", "redis-cache"]
service-full = ["service-observability", "service-distributed"]
```

Notes:
- `service-runtime` is the preferred baseline for service-style consumers.
- `service-observability` adds metrics on top of the runtime layer.
- `service-distributed` adds Redis-backed cache helpers on top of the runtime layer.
- `service-full` is the canonical "all service capabilities" path.
- The older granular flags `service-cache`, `service-metrics`, `service-rate-limiting`, and `redis-cache` remain available as compatibility escape hatches, but they are no longer the recommended public composition model.
- CI now validates the layered feature set instead of trying to chase every theoretical combination.
- This crate already contains a broad set of low-level modules, so consumers should depend on narrower crates when they only need a specific surface.

## Current Position In The Workspace

- `fingerprint-core` is part of the stable workspace surface.
- It is heavily reused by `fingerprint`, `fingerprint-tls`, `fingerprint-http`, `fingerprint-profiles`, `fingerprint-defense`, and `fingerprint-dns`.
- It currently includes some service-oriented modules such as cache, metrics, and rate limiting in addition to pure algorithmic primitives.

## What This README Intentionally Does Not Claim

The following historical concepts are not current public APIs of this crate and are no longer documented here as supported entry points:

- `FingerprintData`
- `BrowserInfo::new(...)` in this crate
- `database` and `connection-pool` Cargo features
- a single all-in-one hash/normalize/validate facade API

## Verification

This crate is part of the default workspace and currently passes workspace `check`, `test --lib`, `fmt`, and `clippy` verification.

## License

BSD-3-Clause. See [LICENSE](../../LICENSE).

---

**Last Updated:** 2026-03-18
