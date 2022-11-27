//! # speculate2
//!
//! An updated fork of `speculate` by
//! [Utkarsh Kukreti](https://github.com/utkarshkukreti/speculate.rs),
//! a crate that provides a very simple macro that is used to easily
//! and elegantly define unit tests in Rust.
//!
//! Please see the documentation for the [`speculate`](./macro.speculate.html) macro
//! for more information and examples.

use std::sync::atomic::{AtomicUsize, Ordering};
use crate::generator::Generate;

mod block;
mod generator;
mod extension;

/// Creates a `test` module using a friendly syntax.
///
/// Inside this block, the following elements can be used:
///
/// * `describe` (or its alias `context`) - to group tests in a hierarchy, for
///   readability. Can be arbitrarily nested.
///
/// * `before` and `after` - contain setup / teardown code that's inserted
///   before / after every sibling and nested `it` block.
///
/// * `it` (or its alias `test`) - contains tests.
///
///   For example:
///
///   ```rust
///   use speculate2::speculate;
///   speculate! {
///       it "can add 1 and 2" {
///           assert_eq!(1 + 2, 3);
///       }
///   }
///   ```
///
///   You can optionally add attributes to this block:
///
///   ```rust
///   use speculate2::speculate;
///   speculate! {
///       #[ignore]
///       test "ignore" {
///           assert_eq!(1, 2);
///       }
///
///       #[should_panic]
///       test "should panic" {
///           assert_eq!(1, 2);
///       }
///
///       #[should_panic(expected = "foo")]
///       test "should panic with foo" {
///           panic!("foo");
///       }
///   }
///   ```
///
/// * Any other Rust "Item", such as `static`, `const`, `fn`, etc.
///
/// # Example
///
/// ```rust
/// use speculate2::speculate;
/// speculate! {
///     const ZERO: i32 = 0;
///
///     fn add(a: i32, b: i32) -> i32 {
///         a + b
///     }
///
///     describe "math" {
///         const ONE: i32 = 1;
///
///         fn sub(a: i32, b: i32) -> i32 {
///             a - b
///         }
///
///         before {
///             let two = ONE + ONE;
///         }
///
///         it "can add stuff" {
///             assert_eq!(ONE, add(ZERO, ONE));
///             assert_eq!(two, add(ONE, ONE));
///         }
///
///         it "can subtract stuff" {
///             assert_eq!(ZERO, sub(ONE, ONE));
///             assert_eq!(ONE, sub(two, ONE));
///         }
///     }
/// }
/// ```
#[proc_macro]
pub fn speculate(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut root = syn::parse2::<block::Root>(input.into()).unwrap();
    root.0.name = {
        static GLOBAL_COUNT: AtomicUsize = AtomicUsize::new(0);
        let invocation_count = GLOBAL_COUNT.fetch_add(1, Ordering::SeqCst);
        let module_name = format!("speculate2_{}", invocation_count);
        syn::Ident::new(&module_name, proc_macro2::Span::call_site())
    };

    let mut output = proc_macro2::TokenStream::new();
    output.extend(quote::quote!( #[allow(non_snake_case)] ));
    output.extend(root.0.generate(None));

    output.into()
}
