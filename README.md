# speculate.rs

An RSpec inspired minimal testing framework for Rust. *(This is a private fork
for Rust 2021, with updated dependencies and removed unstable functionality.)*

[![Build Status][actions-badge]][actions-link]
[![MIT licensed][mit-badge]][mit-link]

[actions-badge]: https://github.com/alexobolev/speculate-rs/actions/workflows/build.yml/badge.svg
[actions-link]: https://github.com/alexobolev/speculate-rs/actions?query=workflow%3ABuild+branch%3Amaster
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-link]: LICENSE.md

## Installation

Add `speculate` to the `dev-dependencies` section of your `Cargo.toml`:

```toml
[dev-dependencies]
speculate = "0.1"
```

And add the following to the top of the Rust file you want to add tests for:

```rust
#[cfg(test)]
use speculate::speculate;  // Must be imported into the current scope.
```

## Usage

Speculate provides the `speculate!` syntax extension.
Inside `speculate! { ... }`, you can have any "Item", like `static`, `const`,
`fn`, etc, and 5 special types of blocks:

* `describe` (or its alias `context`) - to group tests in a hierarchy, for
  readability. Can be arbitrarily nested.

* `before` and `after` - contain setup / teardown code that's inserted
  before / after every sibling and nested `it` block.

* `it` (or its alias `test`) - contains tests.

  For example:

  ```rust
  it "can add 1 and 2" {
      assert_eq!(1 + 2, 3);
  }
  ```

  You can optionally add attributes to this block:

  ```rust
  #[ignore]
  test "ignore" {
      assert_eq!(1, 2);
  }

  #[should_panic]
  test "should panic" {
      assert_eq!(1, 2);
  }

  #[should_panic(expected = "foo")]
  test "should panic with foo" {
      panic!("foo");
  }
  ```

## License

Licensed same as the original repository, under MIT License.
A copy can be found in the repo root.
