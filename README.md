# Rust crate objio

This crate provides simple traits for reading and writing objects.

The traits `ObjectReader` and `ObjectWriter` are **not** intended as a generalized
serialization framework like serde, they are provided to simply read/write
specific object types in specific formats.

## Example

TBD.

## Changes

### Version 0.1.2

* Documentation: added documentation to all traits and a detailed example at
the module level.

### Version 0.1.1

* Refactor: updated error type processing.
  * Removed custom `Error` type.
  * Changed trait Error types to have a constraint requiring
    `From<<std::io::Error>>`.

### Version 0.1.0

* Initial release.
