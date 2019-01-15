# Reducer [![merit.badge]][merit.reducer] [![travis.badge]][travis.home] [![codecov.badge]][codecov.reducer] [![deps.rs.badge]][deps.rs.reducer]

A platform for reactive programming in Rust that can be used to manage the state of
any kind of application. It shines when used to drive graphical user interfaces and
[integrates well with both immediate mode and retained mode GUI frameworks](#examples).

## Using Reducer

Reducer is available on [crates.io], simply add it as a dependency in your `Cargo.toml`:

```
[dependencies]
reducer = "1.0"
```

and import it in your `lib.rs`:

```
extern crate reducer;
use reducer::*;
```

The full API documentation is available on [docs.rs]

## Examples

To see Reducer in action, check out the [examples] directory, which includes a number of
implementations of a simple Todo List app using Reducer to drive various popular GUI frameworks.

To run an example in particular, you can execute

```
> cargo run --release --example <NAME>
```

> **Note to macOS users:** due to an issue with `ui-sys` you might need to prepend
> `CXXFLAGS+=-stdlib=libc++` to the command above, see
> [brunocodutra/reducer#1](https://github.com/brunocodutra/reducer/issues/1).

## Contribution

Reducer is an open source project and you're very welcome to contribute to this project by
opening [issues] and/or [pull requests][pulls], see [CONTRIBUTING][CONTRIBUTING] for general
guidelines.

## License

Reducer is distributed under the terms of the MIT license, see [LICENSE] for details.

[merit.badge]:      http://meritbadge.herokuapp.com/reducer
[merit.reducer]:    https://crates.io/crates/reducer

[travis.home]:      https://travis-ci.org/brunocodutra/reducer
[travis.badge]:     https://travis-ci.org/brunocodutra/reducer.svg?branch=master

[codecov.reducer]:  https://codecov.io/gh/brunocodutra/reducer
[codecov.badge]:    https://codecov.io/gh/brunocodutra/reducer/branch/master/graph/badge.svg

[deps.rs.badge]:    https://deps.rs/repo/github/brunocodutra/reducer/status.svg
[deps.rs.reducer]:  https://deps.rs/repo/github/brunocodutra/reducer

[crates.io]:        https://crates.io/crates/reducer
[docs.rs]:          https://docs.rs/reducer

[issues]:           https://github.com/brunocodutra/reducer/issues
[pulls]:            https://github.com/brunocodutra/reducer/pulls
[examples]:         https://github.com/brunocodutra/reducer/tree/master/examples

[LICENSE]:          https://github.com/brunocodutra/reducer/blob/master/LICENSE
[CONTRIBUTING]:     https://github.com/brunocodutra/reducer/blob/master/CONTRIBUTING.md
