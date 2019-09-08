# Reducer [![crate.badge]][crate.home] [![docs.badge]][docs.home] [![codecov.badge]][codecov.home]

A platform for reactive programming in Rust that can be used to manage the state of
any kind of application. It shines when used to drive graphical user interfaces and
[integrates well with both immediate mode and retained mode GUI frameworks](#examples).

## Using Reducer

Reducer is available on [crates.io][crate.home], simply add it as a dependency in your `Cargo.toml`:

```
[dependencies]
reducer = "2.0"
```

and import it in your `lib.rs`:

```
use reducer::*;
```

The full API documentation is available on [docs.rs][docs.home]

## Examples

To see Reducer in action, check out the [examples] directory.
There you'll find multiple implementations of a simple Todo List app using Reducer to drive popular
GUI frameworks.

To run an example, execute

```
> cargo run --release --example <NAME>
```

where `<NAME>` can be one of [conrod] or [iui].

> **Note to macOS users:** due to an issue with `ui-sys` you might need to prepend
> `CXXFLAGS+=-stdlib=libc++` to the command above, see
> [brunocodutra/reducer#1](https://github.com/brunocodutra/reducer/issues/1).

## Contribution

Reducer is an open source project and you're very welcome to contribute to this project by
opening [issues] and/or [pull requests][pulls], see [CONTRIBUTING][CONTRIBUTING] for general
guidelines.

## License

Reducer is distributed under the terms of the MIT license, see [LICENSE] for details.

[crate.home]:       https://crates.io/crates/reducer
[crate.badge]:      https://meritbadge.herokuapp.com/reducer

[docs.home]:        https://docs.rs/reducer
[docs.badge]:       https://docs.rs/reducer/badge.svg

[codecov.home]:     https://codecov.io/gh/brunocodutra/reducer
[codecov.badge]:    https://codecov.io/gh/brunocodutra/reducer/branch/master/graph/badge.svg

[issues]:           https://github.com/brunocodutra/reducer/issues
[pulls]:            https://github.com/brunocodutra/reducer/pulls
[examples]:         https://github.com/brunocodutra/reducer/tree/master/examples

[conrod]:           https://crates.io/crates/Conrod
[iui]:              https://crates.io/crates/iui

[LICENSE]:          https://github.com/brunocodutra/reducer/blob/master/LICENSE
[CONTRIBUTING]:     https://github.com/brunocodutra/reducer/blob/master/CONTRIBUTING.md
