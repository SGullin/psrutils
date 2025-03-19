# `psrutils`
A small crate for use in pulsar timing projects.

## Parfiles
The `parfile` module takes care of reading, representing, and writing `.par` files. The struct `Parfile` is available with `pub` fields for easy editing, and methods for reading from or writing to buffer objects.

## Timfiles
The `timfile` module contains a method for reading `.tim` files, yielding a `Vec<TOAInfo>`. This recursively reads any other files included with the `INCLUDE` keyword.

# Changelog
## v 0.2.1
 * Added .tim parser (currently only in tempo2-mode).
 * Changed hierarchy, moving error to be crate-global.

## v 0.1.0
 * Added .par parser.
