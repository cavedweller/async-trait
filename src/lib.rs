//! <h5>Type erasure for async trait methods</h5>
//!
//! The async/await language feature is on track for an initial round of
//! stabilizations in Rust 1.38 (tracking issue: [rust-lang/rust#62149]), but
//! this does not include support for async fn in traits. Trying to include an
//! async fn in a trait produces the following error:
//!
//! [rust-lang/rust#62149]: https://github.com/rust-lang/rust/issues/62149
//!
//! ```compile_fail
//! #![feature(async_await)]
//!
//! trait MyTrait {
//!     async fn f() {}
//! }
//! ```
//!
//! ```text
//! error[E0706]: trait fns cannot be declared `async`
//!  --> src/main.rs:4:5
//!   |
//! 4 |     async fn f() {}
//!   |     ^^^^^^^^^^^^^^^
//! ```
//!
//! This crate provides an attribute macro to make async fn in traits work.
//!
//! <br>
//!
//! # Example
//!
//! This example implements the core of a highly effective advertising platform
//! using async fn in a trait.
//!
//! The only thing to notice here is that we write an `#[async_trait]` macro on
//! top of traits and trait impls that contain async fn, and then they work.
//!
//! ```
//! #![feature(async_await)]
//!
//! use async_trait::async_trait;
//!
//! #[async_trait]
//! trait Advertisement {
//!     async fn run(&self);
//! }
//!
//! struct Modal;
//!
//! #[async_trait]
//! impl Advertisement for Modal {
//!     async fn run(&self) {
//!         self.render_fullscreen().await;
//!         for _ in 0..4u16 {
//!             remind_user_to_join_mailing_list().await;
//!         }
//!         self.hide_for_now().await;
//!     }
//! }
//!
//! struct AutoplayingVideo {
//!     media_url: String,
//! }
//!
//! #[async_trait]
//! impl Advertisement for AutoplayingVideo {
//!     async fn run(&self) {
//!         let stream = connect(&self.media_url).await;
//!         stream.play().await;
//!
//!         // Video probably persuaded user to join our mailing list!
//!         Modal.run().await;
//!     }
//! }
//! #
//! # impl Modal {
//! #     async fn render_fullscreen(&self) {}
//! #     async fn hide_for_now(&self) {}
//! # }
//! #
//! # async fn remind_user_to_join_mailing_list() {}
//! #
//! # struct Stream;
//! # async fn connect(_media_url: &str) -> Stream { Stream }
//! # impl Stream {
//! #     async fn play(&self) {}
//! # }
//! ```
//!
//! <br><br>
//!
//! # Supported features
//!
//! ## Non-Send/Sync Async traits
//! Not all async traits need Send/Sync TypeBounds. To avoid having them placed
//! on your generated methods, annotate your traits as `#[async_trait(local)]`
//!
//!
//! It is the intention that all features of Rust traits should work nicely with
//! #\[async_trait\], but the edge cases are numerous. Please file an issue if
//! you see unexpected borrow checker errors, type errors, or warnings. There is
//! no use of `unsafe` in the expanded code, so rest assured that if your code
//! compiles it can't be that badly broken.
//!
//! > &#9745;&emsp;Self by value, by reference, by mut reference, or no self;<br>
//! > &#9745;&emsp;Any number of arguments, any return value;<br>
//! > &#9745;&emsp;Generic type parameters and lifetime parameters;<br>
//! > &#9745;&emsp;Associated types;<br>
//! > &#9745;&emsp;Having async and non-async functions in the same trait;<br>
//! > &#9745;&emsp;Default implementations provided by the trait;<br>
//! > &#9745;&emsp;Elided lifetimes;<br>
//! > &#9745;&emsp;Dyn-capable traits.<br>
//!
//! <br>
//!
//! # Explanation
//!
//! Async fns get transformed into methods that return `Pin<Box<dyn Future +
//! Send + 'async>>` and delegate to a private async freestanding function.
//!
//! For example the `impl Advertisement for AutoplayingVideo` above would be
//! expanded as:
//!
//! ```
//! # const IGNORE: &str = stringify! {
//! impl Advertisement for AutoplayingVideo {
//!     fn run<'async>(
//!         &'async self,
//!     ) -> Pin<Box<dyn core::future::Future<Output = ()> + Send + 'async>>
//!     where
//!         Self: Sync + 'async,
//!     {
//!         async fn run(_self: &AutoplayingVideo) {
//!             /* the original method body */
//!         }
//!
//!         Pin::from(Box::new(run(self)))
//!     }
//! }
//! # };
//! ```
//!
//! <br><br>
//!
//! # Elided lifetimes
//!
//! Be aware that async fn syntax does not allow lifetime elision outside of `&`
//! and `&mut` references. (This is true even when not using #\[async_trait\].)
//! Lifetimes must be named or marked by the placeholder `'_`.
//!
//! Fortunately the compiler is able to diagnose missing lifetimes with a good
//! error message.
//!
//! ```compile_fail
//! # #![feature(async_await)]
//! #
//! # use async_trait::async_trait;
//! #
//! type Elided<'a> = &'a usize;
//!
//! #[async_trait]
//! trait Test {
//!     async fn test(not_okay: Elided, okay: &usize) {}
//! }
//! ```
//!
//! ```text
//! error[E0726]: implicit elided lifetime not allowed here
//!  --> src/main.rs:9:29
//!   |
//! 9 |     async fn test(not_okay: Elided, okay: &usize) {}
//!   |                             ^^^^^^- help: indicate the anonymous lifetime: `<'_>`
//! ```
//!
//! The fix is to name the lifetime or use `'_`.
//!
//! ```
//! # #![feature(async_await)]
//! #
//! # use async_trait::async_trait;
//! #
//! # type Elided<'a> = &'a usize;
//! #
//! #[async_trait]
//! trait Test {
//!     // either
//!     async fn test<'e>(elided: Elided<'e>) {}
//! # }
//! # #[async_trait]
//! # trait Test2 {
//!     // or
//!     async fn test(elided: Elided<'_>) {}
//! }
//! ```
//!
//! <br><br>
//!
//! # Dyn traits
//!
//! Traits with async methods can be used as trait objects as long as they meet
//! the usual requirements for dyn -- no methods with type parameters, no self
//! by value, no associated types, etc.
//!
//! ```
//! # #![feature(async_await)]
//! #
//! # use async_trait::async_trait;
//! #
//! #[async_trait]
//! pub trait ObjectSafe {
//!     async fn f(&self);
//!     async fn g(&mut self);
//! }
//!
//! # const IGNORE: &str = stringify! {
//! impl ObjectSafe for MyType {...}
//!
//! let value: MyType = ...;
//! # };
//! #
//! # struct MyType;
//! #
//! # #[async_trait]
//! # impl ObjectSafe for MyType {
//! #     async fn f(&self) {}
//! #     async fn g(&mut self) {}
//! # }
//! #
//! # let value: MyType = MyType;
//! let object = &value as &dyn ObjectSafe;  // make trait object
//! ```
//!
//! The one wrinkle is in traits that provide default implementations of async
//! methods. In order for the default implementation to produce a future that is
//! Send, the async_trait macro must emit a bound of `Self: Sync` on trait
//! methods that take `&self` and a bound `Self: Send` on trait methods that
//! take `&mut self`. An example of the former is visible in the expanded code
//! in the explanation section above.
//!
//! If you make a trait with async methods that have default implementations,
//! everything will work except that the trait cannot be used as a trait object.
//! Creating a value of type `&dyn Trait` will produce an error that looks like
//! this:
//!
//! ```text
//! error: the trait `Test` cannot be made into an object
//!  --> src/main.rs:8:5
//!   |
//! 8 |     async fn cannot_dyn(&self) {}
//!   |     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
//! ```
//!
//! For traits that need to be object safe and need to have default
//! implementations for some async methods, there are two resolutions. Either
//! you can add Send and/or Sync as supertraits (Send if there are `&mut self`
//! methods with default implementations, Sync if there are `&self` methods with
//! default implementions) to constrain all implementors of the trait such that
//! the default implementations are applicable to them:
//!
//! ```
//! # #![feature(async_await)]
//! #
//! # use async_trait::async_trait;
//! #
//! #[async_trait]
//! pub trait ObjectSafe: Sync {  // added supertrait
//!     async fn can_dyn(&self) {}
//! }
//! #
//! # struct MyType;
//! #
//! # #[async_trait]
//! # impl ObjectSafe for MyType {}
//! #
//! # let value = MyType;
//!
//! let object = &value as &dyn ObjectSafe;
//! ```
//!
//! or you can strike the problematic methods from your trait object by
//! bounding them with `Self: Sized`:
//!
//! ```
//! # #![feature(async_await)]
//! #
//! # use async_trait::async_trait;
//! #
//! #[async_trait]
//! pub trait ObjectSafe {
//!     async fn cannot_dyn(&self) where Self: Sized {}
//!
//!     // presumably other methods
//! }
//! #
//! # struct MyType;
//! #
//! # #[async_trait]
//! # impl ObjectSafe for MyType {}
//! #
//! # let value = MyType;
//!
//! let object = &value as &dyn ObjectSafe;
//! ```

extern crate proc_macro;

mod expand;
mod lifetime;
mod parse;
mod receiver;

use crate::expand::expand;
use crate::parse::Item;
use proc_macro::TokenStream;
use quote::quote;
use syn::parse_macro_input;

mod kw {
    syn::custom_keyword!(local);
}

#[proc_macro_attribute]
pub fn async_trait(args: TokenStream, input: TokenStream) -> TokenStream {
    let local = parse_macro_input!(args as Option<kw::local>).is_some();
    let mut item = parse_macro_input!(input as Item);
    expand(&mut item, local);
    TokenStream::from(quote!(#item))
}
