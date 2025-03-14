// use std::path::{Path, PathBuf};
// use super::error::Result;
//
// pub fn validate_files(paths: Vec<PathBuf>) -> Result<Vec<PathBuf>> {
//     if paths.is_empty() {
//         return Err("No input files provided.".into());
//     }
//
//     let mut files = Vec::new();
//     for f in paths {
//
//         let file = Path::new(f.as_);
//         if file.is_dir() {
//
//             for e in file.read_dir()? {
//                 let e = e?;
//                 let path = e.path();
//                 if path.is_file() {
//                     files.push(path);
//                 }
//             }
//             return Err(format!("Directory provided: {:?}", f).into());
//         }
//         if file.is_file() {
//             files.push(f);
//         }
//     }
//
//     Ok(files)
//
// }

use std::fs;
use std::path::Path;

/// Get the sequence type from the file extension
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
