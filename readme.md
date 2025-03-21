# `psrutils`
[![GitHub license](https://img.shields.io/badge/license-Apache--2.0_OR_MIT-blue)](#license)
[![Crates.io Version](https://img.shields.io/crates/v/psrutils)](https://crates.io/crates/psrutils)
[![GitHub](https://badgen.net/badge/icon/github?icon=github&label)](https://github.com/SGullin/psrutils)
[![docs](https://img.shields.io/docsrs/psrutils?logo=rust&style)](https://docs.rs/psrutils/latest/)
[![docs](https://img.shields.io/docsrs/psrutils?logo=rust&style)](https://docs.rs/psrutils/latest/)


A small crate for use in pulsar timing projects.

## Parfiles
The `parfile` module takes care of reading, representing, and writing `.par` files. The struct `Parfile` is available with `pub` fields for easy editing, and methods for reading from or writing to buffer objects.

## Timfiles
The `timfile` module contains a method for reading `.tim` files, yielding a `Vec<TOAInfo>`. This recursively reads any other files included with the `INCLUDE` keyword.

# Changelog
## v 0.2.2
 * Added module `data_types`, and J2000 coordinate types within.
 * Fixed visibility issues.
 * Added more documentation.

## v 0.2.1
 * Added .tim parser (currently only in tempo2-mode).
 * Changed hierarchy, moving error to be crate-global.

## v 0.1.0
 * Added .par parser.


### License
`psrutils` is distributed under the terms of both the [MIT License](LICENSE-MIT) and the [Apache License (2.0)](LICENSE-APACHE).
