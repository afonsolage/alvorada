# Alvorada

[![Deploy to GitHub Pages](https://github.com/afonsolage/alvorada/actions/workflows/deploy.yml/badge.svg)](https://github.com/afonsolage/alvorada/actions/workflows/deploy.yml)

A 2D game built with [Bevy](https://bevyengine.org/) and Rust, build entirely with Github Coding Agent, for learning purposes.

> **Play online:** https://afonsolage.github.io/alvorada/

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.

## Prerequisites

- [Rust nightly](https://rustup.rs/) (the `rust-toolchain.toml` file will install the correct toolchain automatically)
- For the native build, a C compiler and GPU drivers are required

## Building

### Native

```sh
cargo run
```

### WebAssembly

```sh
# Install wasm-bindgen-cli (version must match Cargo.lock)
cargo install wasm-bindgen-cli --locked

# Build
cargo build --profile wasm-release --target wasm32-unknown-unknown

# Generate WASM bindings
wasm-bindgen \
  --out-dir web \
  --target web \
  --no-typescript \
  target/wasm32-unknown-unknown/wasm-release/alvorada.wasm

# Serve the web/ directory with any static file server
```
