/// Utils
///
/// The module has some useful functions.
///

use std::{fs, io};
use std::fs::File;
use std::path::{Path, PathBuf};
use crate::error::e_exit;
use bio::io::{fasta, fastq, gff};
use bio::io::gff::GffType;

/// Inside defined file types
pub enum FileType {
    Fasta,
    Fastq,
    Gff,
    Unknown,
}

impl FileType {
    /// Infer file type by extension name.
    pub fn infer_file_type(path: &PathBuf) -> FileType {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| match ext.to_lowercase().as_str() {
                "fa" | "fasta" | "pep" => FileType::Fasta,      // NOTE: PEP needs investigation
                "gff" | "gff3" => FileType::Gff,
                "fq" | "fastq" => FileType::Fastq,
                _ => FileType::Unknown
            })
            .unwrap_or(FileType::Unknown)
    }
}


/// Multiple format file writer based on [bio crate] .
pub struct MultiFormatWriter {
    pub fa: fasta::Writer<File>,
    pub fq: fastq::Writer<File>,
    pub gff: gff::Writer<File>,
}

impl MultiFormatWriter {
    pub fn new(path: &PathBuf) -> io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self {
            fa: fasta::Writer::new(file.try_clone()?),
            gff: gff::Writer::new(file.try_clone()?, GffType::GFF3),   // TODO: GFF Type
            fq: fastq::Writer::new(file),
        })
    }
}

/// Get the sequence type from the file extension
///
pub fn try_file_type_ext(file: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let ext = file.extension().unwrap().to_str().unwrap();
    match ext {
        "fasta" | "fa" => Ok("fasta".to_string()),
        "fastq" | "fq" => Ok("fastq".to_string()),
        "gff" | "gtf" => Ok("gff".to_string()),
        "bed" => Ok("bed".to_string()),
        "sam" => Ok("sam".to_string()),
        "bam" => Ok("bam".to_string()),
        _ => Err(format!("Unknown file extension: {:?}", ext).into()),
    }
}

/// Check the sequence type by a fast way: see if some special symbols exist in the sequence
///
pub fn try_seq_type_seq(seq: &[u8]) -> String {
    if seq.is_empty() {
        eprintln!("Empty sequence");
    }

    let (mut is_dna, mut is_rna, mut is_protein) = (true, true, true);
    for &c in seq {
        let c_upper = c.to_ascii_uppercase();
        let mut valid_in_any = false;

        // Check DNA validity
        if is_dna {
            // Some files may contain 'N' as a placeholder for unknown bases
            if matches!(c_upper, b'A' | b'T' | b'C' | b'G' | b'N') {
                valid_in_any = true;
            } else {
                is_dna = false;
            }
        }

        // Check RNA validity
        if is_rna {
            if matches!(c_upper, b'A' | b'U' | b'C' | b'G') {
                valid_in_any = true;
            } else {
                is_rna = false;
            }
        }

        // Check Protein validity
        if is_protein {
            if matches!(
                c_upper,
                b'A' | b'R'
                    | b'N'
                    | b'D'
                    | b'C'
                    | b'E'
                    | b'Q'
                    | b'G'
                    | b'H'
                    | b'I'
                    | b'L'
                    | b'K'
                    | b'M'
                    | b'F'
                    | b'P'
                    | b'S'
                    | b'T'
                    | b'W'
                    | b'Y'
                    | b'V'
                    | b'B'
                    | b'J'
                    | b'O'
                    | b'U'
                    | b'X'
                    | b'Z'
            ) {
                valid_in_any = true;
            } else {
                is_protein = false;
            }
        }

        // Early exit if invalid character
        if !valid_in_any {
            eprintln!("Invalid character: {}", c as char);
        }
        // Early exit if only one type is valid
        if only_one_true(is_dna, is_rna, is_protein) {
            break;
        }
    }

    // Determine result by priority
    // if `is_dna && is_protein` equals to true, the sequence is seen as DNA.
    if is_dna {
        "DNA".into()
    } else if is_rna {
        "RNA".into()
    } else if is_protein {
        "Protein".into()
    } else {
        "Unknown sequence type".into()
    }
}

fn only_one_true(a: bool, b: bool, c: bool) -> bool {
    (a as u8 + b as u8 + c as u8) == 1
}

/// Write `content` into file given by `path`
///
pub fn write_file<P: AsRef<Path>>(path: P, content: &str) {
    fs::write(path, content).expect("Unable to write file");
}

/// See what type(file or dir) does the path behalf.
pub fn is_directory_path(path: &PathBuf) -> bool {
    path.extension().map_or(true, |ext| {
        ext.is_empty() || path.as_os_str().to_str().unwrap().ends_with('.')
    })
}

/// Creat empty file.
///
pub fn create_file_with_dir(path: &Path) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|e| {
            e_exit("DIR", &format!("Unable to create directory: {}", e), 1);
        });
    }

    File::create(path).unwrap_or_else(|e| {
        e_exit("FILE", &format!("Unable to create file: {}", e), 1);
    });
}
