## Guidelines

* All code submitted to Reducer via pull requests is
assumed to be [licensed under the MIT][LICENSE].
* Every code change must be covered by unit tests, use [tarpaulin] to generate the code coverage report:
  + `cargo +nightly tarpaulin -v --all-features`
* Besides `cargo test`, make sure [Clippy] and [rustfmt] checks also pass before submitting a pull request:
  + `cargo clippy --all-targets -- -D warnings`
  + `cargo fmt --all -- --check`

[Clippy]:       https://github.com/rust-lang/rust-clippy#usage
[rustfmt]:      https://github.com/rust-lang/rustfmt#quick-start
[tarpaulin]:    https://github.com/xd009642/tarpaulin#usage
[LICENSE]:      https://github.com/brunocodutra/reducer/blob/master/LICENSE