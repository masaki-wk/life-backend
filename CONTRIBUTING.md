# Contribution Guide

Thank you for taking the time to contribute!  This guide provides an overview of
how to contribute to the project.

All contributors are expected to follow [the Code of Conduct](CODE_OF_CONDUCT.md).

## Links to important resources

- [Code of Conduct](CODE_OF_CONDUCT.md)
- License: [Apache-2.0 license](LICENSE-APACHE) / [MIT license](LICENSE-MIT)
- [the repository on GitHub](https://github.com/masaki-wk/life-backend)
- [the registered crate on crates.io](https://crates.io/crates/life-backend)
- [Documentation on Docs.rs](https://docs.rs/life-backend/latest/life_backend/)
- [README](README.md) for users

## Instructions to contribute

- Reporting bugs, Requesting/suggesting features
  - Please use [Issues](https://github.com/masaki-wk/life-backend/issues)
- Asking questions, discussing topics related to the project
  - Please use [Discussions](https://github.com/masaki-wk/life-backend/discussions)
- Making changes to the source code or documentation
  (all documentation is embedded in the code)
  1. Set up the development environment (as described later)
  2. Create a new topic branch in the repository for the changes.  Refer to
     [GitHub flow](https://docs.github.com/en/get-started/quickstart/github-flow)
     for more details
  3. Make the necessary changes
  4. Create a pull request
     - Please use [Pull requests](https://github.com/masaki-wk/life-backend/pulls)

## How to set up the development environment

This project uses the tools listed below, many of which are common tools for
Rust programming.  Please refer to each link for details.  No platform-specific
tools are required.

- Common Rust development environment, including `rustc` and `cargo`:
  see [Install Rust](https://www.rust-lang.org/tools/install)
- [Rustfmt](https://rust-lang.github.io/rustfmt/)
- [Clippy](https://doc.rust-lang.org/clippy/)
- [rustdoc](https://doc.rust-lang.org/rustdoc/)
- [cargo-readme](https://crates.io/crates/cargo-readme/)

## Required checks for changes

Pull requests to `main` branch must pass the following checks, which are
automatically performed via GitHub Actions.

- Code formatting using Rustfmt should already be applied
- Lint-checks using Clippy should have no errors or warnings
- README.md should be generated from the committed code using cargo-readme
- All tests should pass by executing `cargo test`

Missing doc comments for public items are not allowed because the lint rule
[missing_docs](https://doc.rust-lang.org/rustdoc/lints.html#missing_docs) is
enabled.
