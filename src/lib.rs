//!
//! [![Version](https://img.shields.io/badge/version-0.1.0-yellow.svg)]()
//! [![GitHub](https://img.shields.io/badge/github-bio--here%2Fseq--here-blue.svg)](https://github.com/bio-here/seq-here)
//! [![Build Status](https://travis-ci.org/bio-here/seq-here.svg?branch=master)](https://travis-ci.org/bio-here/seq-here)
//! [![Crates.io](https://img.shields.io/crates/v/seq-here.svg)](https://crates.io/crates/seq-here)
//! [![Documentation](https://docs.rs/seq-here/badge.svg)](https://docs.rs/seq-here)
//! [![License](https://img.shields.io/crates/l/MIT.svg)]()!
//!
//! This crate provides several functions for bio-sequence file processing. It is designed to be fast and easy to use.
//!
//! Use the crate in your project by adding the following to your `Cargo.toml`:
//! ```toml
//! seq-here = "0.1.0"
//! ```
//!
//! There are 3 modules in this crate for different purposes:
//! - **info**: Get basic information about the input sequence file(s).
//! - **process**: Process incoming sequence file(s).
//! - **extract**: Extract specified sequence segment or file data.
//!
//! ## Examples
//!
//! - Info module:
//!
//! ```rust
//! use seq_here::info::{self, InfoOutput};
//! use std::path::{Path, PathBuf};
//!
//! let paths = vec![PathBuf::from("tests/test.fa")];
//! info::InfoFa::by_println(paths.clone());
//! info::InfoFa::by_file(paths);
//! ```
//!
//! - Process module:
//!
//! ```rust
//! use seq_here::process::{self};
//! use std::path::PathBuf;
//!
//! // Combine multiple files into one
//! let input_files = vec![PathBuf::from("file1.txt"), PathBuf::from("file2.txt")];
//! let output_file = PathBuf::from("combined.txt");
//! seq_here::process::ConvertCombine::combine_all(input_files, output_file);
//! ```
//!
//! - Extract module:
//!
//! ```rust
//! use seq_here::extract::{ExtractSegment, ExtractExplain};
//! use std::path::PathBuf;
//!
//! // Extract sequence by ID
//! let input_files = vec![PathBuf::from("sequence.fasta")];
//! let output_file = PathBuf::from("extracted.fasta");
//! let id = "sequence_id".to_string();
//! 
//! // Extract full sequence matching the ID
//! ExtractSegment::extract_id(input_files.clone(), id.clone(), output_file.clone(), None, None);
//! 
//! // Extract a specific segment (positions 10 to 50) from the sequence
//! ExtractSegment::extract_id(input_files, id, output_file, Some(10), Some(50));
//! 
//! // Extract features from annotation files
//! let seq_files = vec![PathBuf::from("genome.fasta")];
//! let anno_files = vec![PathBuf::from("annotations.gff")];
//! let output_dir = PathBuf::from("extracted_features");
//! 
//! // Extract all annotated features
//! ExtractExplain::extract(seq_files.clone(), anno_files.clone(), output_dir.clone(), None);
//! 
//! // Extract only CDS and gene features
//! let feature_types = Some(vec!["CDS".to_string(), "gene".to_string()]);
//! ExtractExplain::extract(seq_files, anno_files, output_dir, feature_types);
//! ```
//!

pub mod process;
pub mod extract;
pub mod info;

pub mod error;
pub mod utils;
