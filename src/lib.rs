//! High-performance wordlist generator for hashcat.
//!
//! `wlgen-rs` is a Rust-based wordlist generator that achieves 100-200M
//! combinations/second, making it ideal for feeding hashcat when cracking
//! password hashes.
//!
//! # Performance
//!
//! - **128-256x faster** than Python implementation (~780K/s)
//! - **100-200M combinations/second** on modern hardware
//! - **O(1) memory usage** - single buffer reused for all words
//! - **Zero-copy iteration** - no string allocations per word
//!
//! # Architecture
//!
//! Uses an "odometer" algorithm similar to hashcat's maskprocessor:
//! - Maintains a single mutable buffer
//! - Increments position indices like an odometer (rightmost first)
//! - Achieves maximum performance through in-place mutation
//!
//! # Example
//!
//! ```
//! use wlgen_rs::WordlistGenerator;
//!
//! let charsets = vec![
//!     b"abc".to_vec(),
//!     b"123".to_vec(),
//! ];
//!
//! let mut gen = WordlistGenerator::new(charsets);
//! for word in gen {
//!     println!("{}", word);
//! }
//! // Prints: a1, a2, a3, b1, b2, b3, c1, c2, c3
//! ```

pub mod cli;
pub mod generator;

pub use cli::Cli;
pub use generator::WordlistGenerator;
