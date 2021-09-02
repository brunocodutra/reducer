# Reducer [![docs.badge]][docs.home] [![codecov.badge]][codecov.home]

A platform for reactive programming in Rust that can be used to manage the state of
any kind of application. It shines when used to drive graphical user interfaces and
[integrates particularly well with immediate mode GUI frameworks](#examples).

## Using Reducer

Reducer is available on [crates.io], simply add it as a dependency in your `Cargo.toml`:

```
[dependencies]
reducer = "3.0"
```

and import it in your `lib.rs`:

```
use reducer::*;
```

The full API documentation is available on [docs.rs][docs.home]

## Examples

The [examples] folder contains the implementation of a simple Todo List app using Reducer
and [Conrod].

```
> cargo run --release --example conrod
```

## Contribution

Reducer is an open source project and you're very welcome to contribute to this project by
opening [issues] and/or [pull requests][pulls], see [CONTRIBUTING][CONTRIBUTING] for general
guidelines.

## License

Reducer is distributed under the terms of the MIT license, see [LICENSE] for details.

[crates.io]:        https://crates.io/crates/reducer

[docs.home]:        https://docs.rs/reducer
[docs.badge]:       https://docs.rs/reducer/badge.svg

[codecov.home]:     https://codecov.io/gh/brunocodutra/reducer
[codecov.badge]:    https://codecov.io/gh/brunocodutra/reducer/branch/master/graph/badge.svg

[Conrod]:           https://crates.io/crates/Conrod

[issues]:           https://github.com/brunocodutra/reducer/issues
[pulls]:            https://github.com/brunocodutra/reducer/pulls
[examples]:         https://github.com/brunocodutra/reducer/tree/master/examples

[LICENSE]:          https://github.com/brunocodutra/reducer/blob/master/LICENSE
[CONTRIBUTING]:     https://github.com/brunocodutra/reducer/blob/master/CONTRIBUTING.md
