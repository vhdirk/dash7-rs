# Dash7-rs

[![Crates.io][crates-badge]][crates-url]
[![Documentation][doc-badge]][doc-url]
[![MIT licensed][mit-badge]][mit-url]
[![codecov][codecov-badge]][codecov-url]
[![ci][ci-badge]][ci-url]

[crates-badge]: https://img.shields.io/crates/v/dash7.svg
[crates-url]: https://crates.io/crates/dash7
[doc-badge]: https://docs.rs/dash7/badge.svg
[doc-url]: https://docs.rs/dash7
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: LICENSE
[codecov-badge]: https://codecov.io/gh/vhdirk/dash7-rs/graph/badge.svg?token=3ATUANHK0O
[codecov-url]: https://codecov.io/gh/vhdirk/dash7-rs
[ci-badge]: https://github.com/vhdirk/dash7-rs/actions/workflows/ci.yml/badge.svg
[ci-url]: https://github.com/vhdirk/dash7-rs/actions/workflows/ci.yml

Dash7 payload encoding and decoding in Rust!

## Library

Currently, this library only assists in encoding and decoding dash7 payloads. Nothing more.
I have plans for supporting `no_std`, possibly even without `alloc`, but as of this writing, only `std` targets are supported.

## CLI

This crate also contains a CLI utility to help you decode dash7 payloads (as hex strings) quickly.

### Installation

Dash7-rs is published on crates.io, so you can just install it with:

```sh
cargo install dash7
```

For [sub-iot](https://github.com/Sub-IoT/Sub-IoT-Stack) payloads

```sh
cargo install dash7 --no-default-features -F std,subiot
```

### Usage

For the help menu, run

```sh
dash7 --help
```

Currently, there's just a single subcommand `parse`:

```sh
dash7 parse --help
```

To parse an ALP payload, you can use:

```sh
dash7 parse -t alp "04 48 00 09 00 00 00 00 00 00 30 00 00 04 48 00 09 00 00 30 00 00 00 00 02 00 04 48 00 09 00 00 70 00 00 00 30 02 00"
```

Parse type and file id are both optional. If neither are given, it will try to parse as any possible type, and as any known system file.
This may however give a false impression of a payload.

## Acknowledgements

Why not <https://github.com/Stratus51/rust_dash7_alp> ? Good question! [@Stratus51](https://github.com/Stratus51) did very good work there. I did, however, dislike that the bit-level operations where so intertwined with the data structs itself. Bit ordering and endianness is hard, especially when host and target differ in endianness.
To that end, this library uses <https://docs.rs/deku>. Deku provides a proc_derive macro for easy field definitions, while the underlying bitvec takes care of all bit-level operations.

However, I have to thank [@Stratus51](https://github.com/Stratus51) for many of the struct defs themselves and even documentation!
