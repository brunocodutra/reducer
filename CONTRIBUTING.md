## Guidelines

* All code submitted to Reducer via pull requests is
assumed to be [licensed under the MIT][LICENSE].
* Every code change must be covered by unit tests, use [tarpaulin] to generate the code coverage report:
  + `cargo +nightly tarpaulin -v --all-features`
* Besides `cargo test`, make sure [Clippy] and [rustfmt] checks also pass before submitting a pull request:
  + `cargo clippy --all-targets -- -D warnings`
  + `cargo fmt --all -- --check`
* Follow [rustsec.org] advisories when introducing new dependencies, use [cargo-audit] to verify:
  + `cargo audit -D`

[LICENSE]:      https://github.com/brunocodutra/reducer/blob/master/LICENSE
[rustsec.org]:  https://rustsec.org/advisories/

[Clippy]:       https://github.com/rust-lang/rust-clippy#usage
[rustfmt]:      https://github.com/rust-lang/rustfmt#quick-start
[tarpaulin]:    https://github.com/xd009642/tarpaulin#usage
[cargo-audit]:  https://github.com/RustSec/cargo-audit#installation
