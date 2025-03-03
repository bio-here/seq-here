//!
//! [![Version](https://img.shields.io/badge/version-0.0.1-yellow.svg)]()
//! [![GitHub](https://img.shields.io/badge/github-bio--here%2Fseq--here-blue.svg)](https://github.com/bio-here/seq-here)
//! [![Build Status](https://travis-ci.org/bio-here/seq-here.svg?branch=master)](https://travis-ci.org/bio-here/seq-here)
//! [![Crates.io](https://img.shields.io/crates/v/seq-here.svg)](https://crates.io/crates/seq-here)
//! [![Documentation](https://docs.rs/seq-here/badge.svg)](https://docs.rs/seq-here)
//! [![License](https://img.shields.io/crates/l/MIT.svg)]()!
//!
//! **Notice**: This project is still under development and not yet ready for production use.
//!
//! This crate provides several functions for bio-sequence file processing. It is designed to be fast and easy to use.
//!
//! Use the crate in your project by adding the following to your `Cargo.toml`:
//! ```toml
//! seq-here = "0.0.1"
//! ```
//!
//! There are 3 modules in this crate for different purposes:
//! - **info**: Get basic information about the input sequence file(s).
//! - **convert**: Convert or process incoming sequence file(s).
//! - **extract**: Extract specified sequence segment or file data.
//!
//!


pub mod info;
pub mod convert;
pub mod extract;

mod error;
pub mod utils;
