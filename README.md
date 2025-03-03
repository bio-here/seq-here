# seq-here

[![Version](https://img.shields.io/badge/version-0.0.1-yellow.svg)]()
[![GitHub](https://img.shields.io/badge/github-bio--here%2Fseq--here-blue.svg)](https://github.com/bio-here/seq-here)
[![Build Status](https://travis-ci.org/bio-here/seq-here.svg?branch=master)](https://travis-ci.org/bio-here/seq-here)
[![Crates.io](https://img.shields.io/crates/v/seq-here.svg)](https://crates.io/crates/seq-here)
[![Documentation](https://docs.rs/seq-here/badge.svg)](https://docs.rs/seq-here)
[![License](https://img.shields.io/crates/l/MIT.svg)]()

A fast tool for bio-sequence file processing.

**NOTICE** : This project is still under development. The current version is 0.0.1.
The usage of this tool is **not** recommended for production yet.


## Installation

You can install `seq-here` using `cargo`:

```shell

cargo install seq-here
```

or you can build it from source:

```shell

git clone [TODO]
cd seq-here
cargo build --release
cp target/release/seq-here /usr/local/bin

seq-here --version
```

## Lib Crate

You can also use `seq-here` as a library crate in your project, 
by adding the following to your `Cargo.toml`:

```toml
[dependencies]
seq-here = "0.0.1"
```


## Usage
To see detailed usage information, you can run:

```shell
seq-here --help
```


- **Info**: Get basic information about the input sequence file(s).

```shell
# Fasta file information
seq-here info fa you_files.fasta

# Fastq file information
seq-here info fq your_files.fastq

# Gff/Gtf file information, Gff2 not supported yet
seq-here info gff your_files.gff
```

- **Convert**: Convert or process incoming sequence file(s).

```shell
[TODO]
```

- **Extract**: Extract specified sequence segment or file data.

```shell
[TODO]
```


## Contributing

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of conduct, and the process for submitting pull requests to us.


## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
