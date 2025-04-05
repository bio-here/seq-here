use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::{fs, io};
use std::io::BufRead;
use crate::error::{e_exit, e_println};
use crate::utils::{FileType, MultiFormatWriter};
use bio::io::gff::GffType;
use bio::io::{fasta, fastq, gff};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;

/// Extract specific segments from biological sequence files
pub struct ExtractSegment;

impl ExtractSegment {
    /// Extract sequences that match a single ID
    /// 
    /// # Arguments
    /// * `paths` - Input sequence files (FASTA, FASTQ, GFF)
    /// * `id` - Sequence identifier to extract
    /// * `output` - Output file path
    pub fn extract_id(paths: Vec<PathBuf>, id: String, output: PathBuf) {
        let id_set = vec![Self::normalize_id(&id)].into_iter().collect();
        Self::process_files_parallel(paths, &id_set, &output)
    }

    /// Extract sequences matching IDs from a file
    /// 
    /// # Arguments
    /// * `paths` - Input sequence files (FASTA, FASTQ, GFF)
    /// * `id_file` - File containing IDs to extract (one per line)
    /// * `output` - Output file path
    pub fn extract_id_files(paths: Vec<PathBuf>, id_file: PathBuf, output: PathBuf) {
        let id_set = match Self::load_id_set(&id_file) {
            Ok(set) => set,
            Err(e) => e_exit("ID-LOAD", &format!("Failed to load IDs: {}", e), 1),
        };
        Self::process_files_parallel(paths, &id_set, &output)
    }

    /// Process multiple files in parallel
    fn process_files_parallel(paths: Vec<PathBuf>, id_set: &HashSet<String>, output: &PathBuf) {
        let writer = match MultiFormatWriter::new(output) {
            Ok(w) => Arc::new(Mutex::new(w)),
            Err(e) => e_exit("WRITER", &format!("Output init failed: {}", e), 2),
        };

        paths.par_iter().for_each(|path| {
            let writer = Arc::clone(&writer);
            match FileType::infer_file_type(path) {
                FileType::Fasta => Self::process_file(path, id_set, writer, Self::process_fasta),
                FileType::Gff => Self::process_file(path, id_set, writer, Self::process_gff),
                FileType::Fastq => Self::process_file(path, id_set, writer, Self::process_fastq),
                FileType::Unknown => e_println("TYPE-ERROR", &format!("Unsupported format: {:?}", path)),
            };
        });
    }

    /// Process a single file with the appropriate processor function
    fn process_file<P>(
        path: &PathBuf,
        ids: &HashSet<String>,
        writer: Arc<Mutex<MultiFormatWriter>>,
        processor: P,
    ) where
        P: Fn(&PathBuf, &HashSet<String>, &mut MultiFormatWriter),
    {
        let mut writer = writer.lock().unwrap_or_else(|e| {
            e_exit("LOCK-ERROR", &format!("Writer lock failed: {}", e), 3)
        });

        processor(path, ids, &mut writer);
    }

    /// Load sequence IDs from a file into a HashSet
    fn load_id_set(path: &PathBuf) -> io::Result<HashSet<String>> {
        let file = File::open(path).map_err(|e| {
            e_println("FILE-ERROR", &format!("Open {} failed: {}", path.display(), e));
            e
        })?;

        let mut set = HashSet::new();
        for line in io::BufReader::new(file).lines() {
            let raw_id = line?.trim().to_string();
            if !raw_id.is_empty() {
                set.insert(Self::normalize_id(&raw_id));
            }
        }
        Ok(set)
    }

    /// Normalize sequence identifiers for consistent matching
    /// 
    /// Takes the first part before any whitespace, pipe, or semicolon
    /// and converts to lowercase for case-insensitive matching
    fn normalize_id(raw_id: &str) -> String {
        raw_id.split(|c: char| c.is_whitespace() || c == '|' || c == ';')
            .next()
            .unwrap_or(raw_id)
            .to_lowercase()
    }

    /// Process FASTA format files to extract matching sequences
    fn process_fasta(path: &PathBuf, ids: &HashSet<String>, writer: &mut MultiFormatWriter) {
        let reader = fasta::Reader::from_file(path)
            .expect(&format!("Failed to open FASTA file: {}", path.display()));
            
        for record in reader.records() {
            let record = record
                .expect(&format!("Failed to parse FASTA record in {}", path.display()));
                
            if ids.contains(&Self::normalize_id(record.id())) {
                writer.fa.write_record(&record)
                    .expect(&format!("Failed to write FASTA record: {}", record.id()));
            }
        }
    }

    /// Process GFF format files to extract matching annotations
    fn process_gff(path: &PathBuf, ids: &HashSet<String>, writer: &mut MultiFormatWriter) {
        let mut reader = gff::Reader::from_file(path, GffType::GFF3)
            .expect(&format!("Failed to open GFF file: {}", path.display()));
            
        for record in reader.records() {
            let record = record
                .expect(&format!("Failed to parse GFF record in {}", path.display()));
                
            if let Some(id) = record.attributes().get("ID") {
                if ids.contains(&Self::normalize_id(id)) {
                    writer.gff.write(&record)
                        .expect(&format!("Failed to write GFF record: {}", id));
                }
            }
        }
    }

    /// Process FASTQ format files to extract matching sequences
    fn process_fastq(path: &PathBuf, ids: &HashSet<String>, writer: &mut MultiFormatWriter) {
        let reader = fastq::Reader::from_file(path)
            .expect(&format!("Failed to open FASTQ file: {}", path.display()));
            
        for record in reader.records() {
            let record = record
                .expect(&format!("Failed to parse FASTQ record in {}", path.display()));
                
            if ids.contains(&Self::normalize_id(record.id())) {
                writer.fq.write_record(&record)
                    .expect(&format!("Failed to write FASTQ record: {}", record.id()));
            }
        }
    }
}

/// Extract and explain annotated sequence features
pub struct ExtractExplain;

impl ExtractExplain {
    /// Extract annotated features from sequences
    /// 
    /// # Arguments
    /// * `seq_files` - FASTA files containing sequences
    /// * `anno_files` - GFF files containing annotations
    /// * `output` - Output directory for extracted features
    pub fn extract(seq_files: Vec<PathBuf>, anno_files: Vec<PathBuf>, output: PathBuf) {
        // Create output directory
        fs::create_dir_all(&output).unwrap_or_else(|e| {
            e_exit("FS", &format!("Failed to create output directory: {}", e), 1);
        });

        // Process each sequence file in parallel
        seq_files.par_iter().for_each(|seq_path| {
            // Load sequence data
            let seq_data = Self::load_sequences(seq_path)
                .unwrap_or_else(|e| e_exit("SEQ-LOAD", &e, 2));

            // Process all annotation files
            let annotations: Vec<_> = anno_files.par_iter()
                .flat_map(|anno_path| {
                    Self::load_annotations(anno_path)
                        .unwrap_or_else(|e| e_exit("ANN-LOAD", &e, 3))
                })
                .collect();

            // Generate result file
            let output_path = output.join(seq_path.file_name().unwrap());
            Self::generate_annotated_file(&seq_data, &annotations, &output_path)
                .unwrap_or_else(|e| e_exit("OUTPUT", &e, 4));
        });
    }

    /// Load sequence data into memory (suitable for small to medium files)
    /// 
    /// Returns a map of sequence IDs to FASTA records
    fn load_sequences(path: &Path) -> Result<HashMap<String, fasta::Record>, String> {
        let reader = fasta::Reader::from_file(path)
            .map_err(|e| format!("Failed to read sequence file: {} - {}", path.display(), e))?;

        let mut seq_map = HashMap::new();
        for record in reader.records() {
            let record = record.map_err(|e| format!("Failed to parse FASTA: {}", e))?;
            seq_map.insert(record.id().to_string(), record);
        }
        Ok(seq_map)
    }

    /// Load GFF annotations
    /// 
    /// Returns a vector of GFF records
    fn load_annotations(path: &Path) -> Result<Vec<gff::Record>, String> {
        let mut reader = gff::Reader::from_file(path, GffType::GFF3)
            .map_err(|e| format!("Failed to read annotation file: {} - {}", path.display(), e))?;

        reader.records()
            .map(|r| r.map_err(|e| format!("Failed to parse GFF: {}", e)))
            .collect()
    }

    /// Generate annotated sequence files
    /// 
    /// Extracts sequence segments based on annotations and writes to output file
    fn generate_annotated_file(
        seq_data: &HashMap<String, fasta::Record>,
        annotations: &[gff::Record],
        output: &Path
    ) -> Result<(), String> {
        let mut writer = fasta::Writer::new(File::create(output)
            .map_err(|e| format!("Failed to create output file: {} - {}", output.display(), e))?);

        // Generate feature sequences for each annotation
        for ann in annotations {
            let seq_id = ann.seqname();
            let Some(seq) = seq_data.get(seq_id) else {
                e_println("ANN-SKIP", &format!("Sequence not found: {}", seq_id));
                continue;
            };

            // Extract sequence for the annotated region
            let feature_seq = Self::extract_feature(seq, ann)
                .map_err(|e| format!("Failed to extract feature: {}", e))?;

            // Generate description
            let description = format!("{}:{}-{} {}",
                                      ann.feature_type(),
                                      ann.start(),
                                      ann.end(),
                                      ann.attributes().get("ID").unwrap_or(&"unknown".to_string())
            );

            // Write new record
            let new_record = fasta::Record::with_attrs(
                ann.attributes().get("ID").unwrap_or(&ann.seqname().to_string()),
                Some(&description),
                &feature_seq
            );
            writer.write_record(&new_record)
                .map_err(|e| format!("Write failed: {}", e))?;
        }
        Ok(())
    }

    /// Extract sequence segment for a feature
    /// 
    /// Extracts the subsequence corresponding to the annotation coordinates
    fn extract_feature(seq: &fasta::Record, ann: &gff::Record) -> Result<Vec<u8>, String> {
        // Convert from 1-based GFF coordinates to 0-based index
        let start = ann.start().saturating_sub(1).to_owned();
        let end = ann.end().to_owned();

        // Validate coordinates
        if start >= seq.seq().len() as u64 || end > seq.seq().len() as u64 {
            return Err(format!("Invalid range: {}-{} (sequence length: {})",
                               ann.start(), ann.end(), seq.seq().len()));
        }

        Ok(seq.seq()[start as usize..end as usize].to_vec())
    }
}
