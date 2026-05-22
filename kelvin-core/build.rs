fn main() {
    // Tell rustc that `cfg(kani)` is a valid conditional compilation
    // option. This suppresses the `unexpected_cfgs` warning when
    // building without Kani.
    println!("cargo::rustc-check-cfg=cfg(kani)");
}
