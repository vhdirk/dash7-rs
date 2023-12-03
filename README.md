# Dash7-rs

[![MIT licensed][mit-badge]][mit-url]
[![codecov][codecov-badge]][codecov-url]

[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: LICENSE
[codecov-badge]: https://codecov.io/gh/vhdirk/dash7-rs/graph/badge.svg?token=3ATUANHK0O
[codecov-url]: https://codecov.io/gh/vhdirk/dash7-rs

An attempt to write the dash7 stack in Rust

Why not <https://github.com/Stratus51/rust_dash7_alp> ? Good question! @Stratus51 did very good work there. But I disliked that the bit-level operations where so intertwined with the data structs itself. So I have opted to use <https://docs.rs/deku> for that.

Howeverl, I have to thank @Stratus51 for many of the struct defs themselves and even documentation!
