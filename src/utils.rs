/// Utils
///
/// The module has some useful functions for bioinformatics file handling and sequence analysis.
///

use std::{fs, io};
use std::fs::File;
use std::path::{Path, PathBuf};
use crate::error::e_exit;
use bio::io::{fasta, fastq, gff};
use bio::io::gff::GffType;

/// Enumeration of supported bioinformatics file types
/// Used for file type detection and handling
pub enum FileType {
    Fasta,  // FASTA sequence files (.fa, .fasta)
    Fastq,  // FASTQ sequence files (.fq, .fastq)
    Gff,    // GFF annotation files (.gff, .gff3)
    Unknown, // Unrecognized file format
}

impl FileType {
    /// Infers the biological file type based on the file extension.
    /// 
    /// # Arguments
    /// * `path` - PathBuf pointing to the file to analyze
    ///
    /// # Returns
    /// * `FileType` enum representing the detected file type
    pub fn infer_file_type(path: &PathBuf) -> FileType {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| match ext.to_lowercase().as_str() {
                "fa" | "fasta" | "pep" => FileType::Fasta,      // DNA/protein sequence files
                "gff" | "gff3" => FileType::Gff,                // Gene feature format
                "fq" | "fastq" => FileType::Fastq,              // Sequence with quality scores
                _ => FileType::Unknown
            })
            .unwrap_or(FileType::Unknown)
    }
}


/// Multiple format file writer based on [bio crate].
/// Provides a unified interface for writing different bioinformatics file formats.
pub struct MultiFormatWriter {
    pub fa: fasta::Writer<File>,  // For writing FASTA format files
    pub fq: fastq::Writer<File>,  // For writing FASTQ format files
    pub gff: gff::Writer<File>,   // For writing GFF/GTF format files
}

impl MultiFormatWriter {
    /// Creates a new MultiFormatWriter that can write to different biological file formats.
    ///
    /// # Arguments
    /// * `path` - PathBuf indicating where to create the output file
    ///
    /// # Returns
    /// * `io::Result<Self>` - The writer instance or an IO error
    pub fn new(path: &PathBuf) -> io::Result<Self> {
        let file = File::create(path)?;
        Ok(Self {
            fa: fasta::Writer::new(file.try_clone()?),
            gff: gff::Writer::new(file.try_clone()?, GffType::GFF3),   // Default to GFF3 format
            fq: fastq::Writer::new(file),
        })
    }
}

/// Determines file type based on file extension
///
/// # Arguments
/// * `file` - Path to the file to analyze
///
/// # Returns
/// * `Result<String, Box<dyn std::error::Error>>` - String representation of file type or an error
pub fn try_file_type_ext(file: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let ext = file.extension().unwrap().to_str().unwrap();
    match ext {
        "fasta" | "fa" => Ok("fasta".to_string()),  // FASTA sequence files
        "fastq" | "fq" => Ok("fastq".to_string()),  // FASTQ sequence files
        "gff" | "gtf" => Ok("gff".to_string()),     // Gene annotation files
        "bed" => Ok("bed".to_string()),             // Browser Extensible Data format
        "sam" => Ok("sam".to_string()),             // Sequence Alignment/Map format
        "bam" => Ok("bam".to_string()),             // Binary version of SAM
        _ => Err(format!("Unknown file extension: {:?}", ext).into()),
    }
}

/// Determines the biological sequence type by analyzing its content
///
/// Uses a heuristic approach to check if the sequence conforms to DNA, RNA, or protein alphabets.
/// In case of ambiguity, prioritizes DNA > RNA > Protein classification.
///
/// # Arguments
/// * `seq` - Byte slice containing the sequence to analyze
///
/// # Returns
/// * `String` - The determined sequence type ("DNA", "RNA", "Protein", or "Unknown")
pub fn try_seq_type_seq(seq: &[u8]) -> String {
    if seq.is_empty() {
        eprintln!("Empty sequence");
    }

    // Track validity flags for each sequence type
    let (mut is_dna, mut is_rna, mut is_protein) = (true, true, true);
    
    for &c in seq {
        let c_upper = c.to_ascii_uppercase();
        let mut valid_in_any = false;

        // Check DNA validity - valid chars are A, T, C, G, N
        if is_dna {
            // N is commonly used as a placeholder for unknown nucleotides
            if matches!(c_upper, b'A' | b'T' | b'C' | b'G' | b'N') {
                valid_in_any = true;
            } else {
                is_dna = false;
            }
        }

        // Check RNA validity - valid chars are A, U, C, G
        if is_rna {
            if matches!(c_upper, b'A' | b'U' | b'C' | b'G') {
                valid_in_any = true;
            } else {
                is_rna = false;
            }
        }

        // Check Protein validity - standard amino acid codes
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
        
        // Optimization: Early exit if only one type remains valid
        if only_one_true(is_dna, is_rna, is_protein) {
            break;
        }
    }

    // Determine result by priority: DNA > RNA > Protein
    // If sequence could be multiple types (e.g., is_dna && is_protein),
    // we classify as the highest priority type
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

/// Utility function that returns true only if exactly one of the three boolean parameters is true
///
/// # Arguments
/// * `a`, `b`, `c` - Three boolean values to check
///
/// # Returns
/// * `bool` - True if exactly one parameter is true, false otherwise
fn only_one_true(a: bool, b: bool, c: bool) -> bool {
    (a as u8 + b as u8 + c as u8) == 1
}

/// Writes string content to a file at the specified path
///
/// # Arguments
/// * `path` - Path where the file should be written
/// * `content` - String content to write to the file
pub fn write_file<P: AsRef<Path>>(path: P, content: &str) {
    fs::write(path, content).expect("Unable to write file");
}

/// Determines if a path represents a directory
///
/// This function uses heuristics based on file extension to guess if the path
/// is meant to be a directory rather than actually checking the filesystem.
///
/// # Arguments
/// * `path` - PathBuf to analyze
///
/// # Returns
/// * `bool` - True if the path likely represents a directory, false otherwise
pub fn is_directory_path(path: &PathBuf) -> bool {
    path.extension().map_or(true, |ext| {
        ext.is_empty() || path.as_os_str().to_str().unwrap().ends_with('.')
    })
}

/// Creates an empty file and ensures its parent directories exist
///
/// # Arguments
/// * `path` - Path where the file should be created
pub fn create_file_with_dir(path: &Path) {
    // First ensure parent directories exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap_or_else(|e| {
            e_exit("DIR", &format!("Unable to create directory: {}", e), 1);
        });
    }

    // Then create the file
    File::create(path).unwrap_or_else(|e| {
        e_exit("FILE", &format!("Unable to create file: {}", e), 1);
    });
}
