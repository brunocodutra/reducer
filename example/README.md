# Examples

Here you'll find multiple implementations of a simple Todo List app using Reducer to drive popular
GUI frameworks.

To run an example, execute

```
> cargo run -p example --release --bin <NAME>
```

where `<NAME>` can be one of [conrod] or [iui].

> **Note to macOS users:** due to an issue with `ui-sys` you might need to prepend
> `CXXFLAGS+=-stdlib=libc++` to the command above, see
> [brunocodutra/reducer#1](https://github.com/brunocodutra/reducer/issues/1).

[conrod]:   https://crates.io/crates/Conrod
[iui]:      https://crates.io/crates/iui