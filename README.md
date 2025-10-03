# OpenIGTLink Rust

A Rust implementation of the [OpenIGTLink](http://openigtlink.org/) protocol for image-guided therapy.

## Overview

OpenIGTLink is an open network protocol for image-guided therapy environments. This library provides a type-safe, performant Rust implementation compatible with the official C++ library.

## Features

- 🦀 **Type-safe**: Leverages Rust's type system for protocol correctness
- 🚀 **Performance**: Zero-copy parsing and efficient serialization
- 🔒 **Memory-safe**: No memory leaks or buffer overflows
- 🔄 **Async/Sync**: Supports both synchronous and asynchronous I/O
- ✅ **Protocol compliance**: Full compatibility with OpenIGTLink Version 2 and 3

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
openigtlink-rust = "0.1.0"
```

## Supported Message Types

- [x] TRANSFORM
- [x] STATUS
- [x] CAPABILITY
- [ ] IMAGE
- [ ] POSITION
- [ ] And more...

## Protocol Specification

This implementation follows the official OpenIGTLink protocol specification:
- Protocol Version: 2 and 3
- Header Size: 58 bytes
- Byte Order: Big-endian
- CRC: 64-bit (compatible with C++ implementation)

## Development Status

This project is currently in active development. The basic infrastructure and core message types are being implemented.

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

## References

- [OpenIGTLink Official Website](http://openigtlink.org/)
- [OpenIGTLink C++ Library](https://github.com/openigtlink/OpenIGTLink)
- [Protocol Specification](https://github.com/openigtlink/OpenIGTLink/blob/master/Documents/Protocol/index.md)
