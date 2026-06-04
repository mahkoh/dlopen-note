# dlopen-note

[![crates.io](https://img.shields.io/crates/v/dlopen-note.svg)](http://crates.io/crates/dlopen-note)
[![docs.rs](https://docs.rs/dlopen-note/badge.svg)](http://docs.rs/dlopen-note)
![MSRV](https://img.shields.io/crates/msrv/dlopen-note)

This crate provides the `dlopen_note` proc macro that can be used to attach dlopen
metadata to an ELF binary.

> Using dlopen() to load optional dependencies brings several advantages:
> programs can gracefully downgrade a feature when a library is not available,
> and the shared library is only loaded into the process (and its ELF
> constructors are run) only when the requested feature is actually used. But
> it also has some drawbacks, and the main one is that it is harder to track a
> program’s dependencies, since unlike build-time dynamic linking there will
> not be a mention in the ELF metadata. This specification aims to solve this
> problem by providing a standardized specification for a custom ELF note that
> can be used to list dlopen() dependencies.

See <https://uapi-group.org/specifications/specs/elf_dlopen_metadata>.

## Example

```rust
use dlopen_note::dlopen_note;

dlopen_note! {
    soname: ["libvulkan.so.1"],
    feature: "vulkan",
    description: "required for the vulkan renderer",
    priority: "recommended",
}
```

## MSRV

The MSRV is `stable - 3`.

## License

This project is licensed under either of

- Apache License, Version 2.0
- MIT License

at your option.
