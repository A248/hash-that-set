
# Hash That Set

[![crates.io version](https://img.shields.io/crates/v/hash-that-set)](https://crates.io/crates/hash-that-set)
[![apache2 license](https://img.shields.io/crates/l/hash-that-set)](https://www.gnu.org/licenses/license-recommendations.html)
[![docs.rs docs](https://img.shields.io/docsrs/hash-that-set)](https://docs.rs/hash-that-set)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)

Implements hashing for `HashSet` or `HashMap` themselves. Enables a map of maps to values, or a map of sets to values.

## Library Usage

Wherever a hashable `HashSet` or `HashMap` is needed, wrap it in a `SumHashes`.

If you have unordered collections from third-party crates, wrap them in `SumHashesAnyCollection`, which uses the default hasher per-element.

### Safety

* The library contains no unsafe code
* The library should never panic

## Dependency

Add this library to your Cargo.toml:

```toml
[dependencies]
hash-that-set = "0.1"
```

## Notes

If you ever happen to notice a place where a standard trait could be implemented, please open an issue or PR in this repository.

### Licensing

Licensed under the Apache License v2.0. See the LICENSE.txt.
