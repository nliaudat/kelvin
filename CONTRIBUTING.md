# Contributing to Kelvin

Thank you for your interest in contributing to Kelvin! As a cryptographic system, we maintain high standards for code quality and security.

## How to Contribute

1.  **Check the Issues**: Look for existing issues or open a new one to discuss your proposed changes.
2.  **Fork the Repo**: Create your own fork and work on a feature branch.
3.  **Follow Coding Standards**:
    *   No `unsafe` code unless absolutely necessary (FFI boundaries).
    *   No floating-point math in core crates.
    *   All public functions must be documented.
    *   All new features must include unit and integration tests.
4.  **Run the Tests**:
    ```bash
    cargo test --all
    cargo clippy --all
    cargo fmt --all -- --check
    cargo deny check
    ```
    If working on feature-gated code (e.g., `subtle-ct`, `failpoints`), test with:
    ```bash
    cargo test --all-features
    ```
5.  **Submit a Pull Request**: Provide a clear description of the changes and link to any relevant issues.

## Security-Sensitive Changes

If you are proposing changes to the core cryptographic logic (Phase 1 simulation, Phase 2 stream cipher, or Key Schedule), please expect a rigorous review process. We may require formal verification (Kani) or side-channel analysis (dudect) results.

For security vulnerabilities, see [SECURITY.md](SECURITY.md) for reporting guidelines.

## Code of Conduct

Please be respectful and professional in all communications. We follow the standard Contributor Covenant.

## License

By contributing, you agree that your contributions will be licensed under the Apache-2.0 / MIT dual license.
