# fingerprint

`fingerprint` is the facade crate for the `fingerprint-rust` workspace. It does not expose a monolithic orchestration engine today. Instead, it re-exports the stable public surface from the profile, TLS, header, HTTP, DNS, and defense crates.

## What This Crate Provides

- random browser fingerprint selection
- browser profile catalog access
- TLS and HTTP helper re-exports
- optional DNS, defense, export, and API-noise integrations

## Quick Start

```toml
[dependencies]
fingerprint = "2.1"
```

```rust
use fingerprint::{get_random_fingerprint, mapped_tls_clients};

let result = get_random_fingerprint().unwrap();
println!("Profile: {}", result.profile_id);
println!("User-Agent: {}", result.user_agent);

let profiles = mapped_tls_clients();
let chrome = profiles.get("chrome_133").unwrap();
println!("Profile ID: {}", chrome.id());
println!("Cipher suites: {}", chrome.tls_config.cipher_suites.len());
println!("HTTP/2 settings: {}", chrome.http2_settings.len());
```

## Public Surface

### Re-exported helpers

- `get_random_fingerprint`
- `get_random_fingerprint_with_os`
- `get_random_fingerprint_by_browser`
- `mapped_tls_clients`
- `generate_headers`
- `get_user_agent_by_profile_name`

### Re-exported client types

- `HttpClient`
- `HttpClientConfig`
- `HttpRequest`
- `HttpResponse`
- `ProxyConfig`
- `CookieStore`

### Optional feature-gated exports

- `dns`: DNS service and resolver helpers
- `defense`: passive analysis and defense helpers
- `api-noise`: API noise module alias
- `export`: serialization/export helpers

## Features

Actual Cargo features for this crate:

```toml
[features]
default = ["rustls-tls", "compression", "http2"]
rustls-tls = ["fingerprint-http/rustls-tls"]
compression = ["fingerprint-http/compression"]
http2 = ["fingerprint-http/http2"]
http3 = ["fingerprint-http/http3"]
async = ["fingerprint-http/async"]
connection-pool = ["fingerprint-http/connection-pool"]
reporter = ["fingerprint-http/reporter"]
export = ["serde", "serde_json", "hex"]
crypto = ["fingerprint-tls/crypto", "fingerprint-http/crypto"]
dangerous_configuration = ["fingerprint-http/dangerous_configuration"]
rustls-client-hello-customizer = ["fingerprint-http/rustls-client-hello-customizer"]
dns = ["fingerprint-dns", "fingerprint-http/rustls-tls"]
defense = ["fingerprint-defense"]
api-noise = ["fingerprint-api-noise"]
```

## Recommended Usage

- Use `fingerprint` when you want the simplest stable entry point.
- Use `fingerprint-profiles`, `fingerprint-tls`, or `fingerprint-http` directly when you need lower-level control.
- Enable `dns`, `defense`, or `api-noise` only when you need those integrations.

## What This Crate Does Not Provide

The following historical concepts are not current public APIs of this crate and should not be used in new documentation or examples:

- `FingerprintEngine`
- `BrowserInfo` as a high-level request aggregation struct in this crate
- `full` or `lightweight` features
- request-to-fingerprint orchestration APIs such as `generate(info)` or `identify(req)`

## Related Crates

- `fingerprint-core`: core algorithms and types
- `fingerprint-tls`: TLS configuration and handshake building
- `fingerprint-http`: HTTP client and protocol support
- `fingerprint-profiles`: browser profile catalog
- `fingerprint-headers`: User-Agent and header generation

## Verification

This crate is part of the default workspace and currently passes workspace `check`, `test --no-run`, `test --lib`, `fmt`, and `clippy` verification.

## License

BSD-3-Clause. See [LICENSE](../../LICENSE).

---

**Last Updated:** 2026-03-18
