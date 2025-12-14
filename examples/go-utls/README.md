# Go uTLS Integration Example

This example demonstrates how to use `fingerprint-rust` to generate a TLS configuration and use it in a Go program with `uTLS`.

## Prerequisites

- Rust (cargo)
- Go 1.20+

## Usage

1. **Generate Configuration**

   Use the `export_config` example from the Rust project to generate a JSON configuration for a specific browser profile (e.g., `chrome_133`).

   ```bash
   # In the root of the Rust project
   cargo run --example export_config chrome_133 examples/go-utls/config.json
   ```

2. **Run Go Client**

   Run the Go program, pointing it to the configuration file and a target URL.

   ```bash
   cd examples/go-utls
   go run main.go config.json https://httpbin.org/get
   ```

## Notes

- This example handles key generation for standard curves (X25519, P256, P384).
- Post-quantum curves (e.g., X25519MLKEM768) are currently filtered out to prevent handshake failures (as the standard Go library doesn't support them yet).
- Encrypted Client Hello (ECH) is currently skipped.
- Protocol violation checks (like PSK dependency) are handled by skipping extensions that lack their dependencies.
