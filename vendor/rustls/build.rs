/// Build script for rustls.
///
/// We use this to:
/// - Declare custom cfg names used by rustls (`read_buf`, `bench`) for `check-cfg` (Rust 1.77+)
/// - Optionally enable `cfg(read_buf)` on nightly when the `read_buf` feature is enabled.
fn main() {
    // Silence `unexpected_cfgs` warnings on stable by declaring these cfg names.
    println!("cargo:rustc-check-cfg=cfg(read_buf)");
    println!("cargo:rustc-check-cfg=cfg(bench)");

    // Enable `cfg(read_buf)` only on nightly when the cargo feature is enabled.
    #[cfg(feature = "read_buf")]
    {
        if rustversion::cfg!(nightly) {
            println!("cargo:rustc-cfg=read_buf");
        }
    }
}
