## Guidelines

* All code submitted to Reducer via pull requests is
assumed to be [licensed under the MIT][LICENSE].
* Every code change must be covered by unit tests, use [cargo-llvm-cov] to generate the code coverage report:
  + `cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info`
* Besides `cargo test`, make sure [Clippy] and [rustfmt] checks also pass before submitting a pull request:
  + `cargo clippy --workspace --all-targets -- -D warnings`
  + `cargo fmt --all -- --check`
* Follow [rustsec.org] advisories when introducing new dependencies, use [cargo-audit] to verify:
  + `cargo audit -D`

[LICENSE]:      https://github.com/brunocodutra/reducer/blob/master/LICENSE
[rustsec.org]:    https://rustsec.org/advisories/

[Clippy]:         https://github.com/rust-lang/rust-clippy#usage
[rustfmt]:        https://github.com/rust-lang/rustfmt#quick-start
[cargo-llvm-cov]: https://github.com/taiki-e/cargo-llvm-cov#usage
[cargo-audit]:    https://github.com/RustSec/cargo-audit#installation
