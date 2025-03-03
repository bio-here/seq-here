use std::path::PathBuf;
use bio::io::fasta;
use bio::io::gff::GffType;
use crate::utils;

/// Define the trait for the different file types
/// 2 ways to output the information: by file or by println
pub trait InfoOutput {
    fn by_file(paths: Vec<PathBuf>);
    fn by_println(paths: Vec<PathBuf>);
}

pub struct InfoFa;
impl InfoOutput for InfoFa {
    fn by_file(paths: Vec<PathBuf>) {
        info_fa(paths);
    }

    fn by_println(paths: Vec<PathBuf>) {
        info_fa(paths);
    }
}

pub struct InfoFq;
impl InfoOutput for InfoFq {
    fn by_file(paths: Vec<PathBuf>) {
        info_fastq(paths);
    }

    fn by_println(paths: Vec<PathBuf>) {
        info_fastq(paths);
    }
}

pub struct InfoGff;
impl InfoOutput for InfoGff {
    fn by_file(paths: Vec<PathBuf>) {
        info_gff(paths, GffType::GFF3);
    }

    fn by_println(paths: Vec<PathBuf>) {
        info_gff(paths, GffType::GFF3);
    }
}


fn info_fa(paths: Vec<PathBuf>) -> String {
    for path in paths {
        let reader = fasta::Reader::from_file(path).unwrap();
        for record in reader.records() {
            let record = record.unwrap();
            println!("Seq Type: {}", utils::try_seq_type_seq(record.seq()).unwrap());
            println!("ID: {}", record.id());
            println!("Description: {}", record.desc().unwrap_or("None"));
            println!("Sequence: {}", String::from_utf8(record.seq().to_vec()).unwrap());
            println!("Length: {}", record.seq().len());
            println!("GC content: {:.2}%", bio::seq_analysis::gc::gc_content(&*record.seq()));
        }
    }

    "Done".to_string()
    
}


pub fn info_fastq(paths: Vec<PathBuf>) {
    for path in paths {
        let reader = bio::io::fastq::Reader::from_file(path).unwrap();
        for record in reader.records() {
            let record = record.unwrap();
            println!("ID: {}", record.id());
            println!("Description: {}", record.desc().unwrap());
            println!("Sequence: {}", String::from_utf8(record.seq().to_vec()).unwrap());
            println!("Quality: {}", String::from_utf8(record.qual().to_vec()).unwrap());
        }
    }
    
}

pub fn info_gff(paths: Vec<PathBuf>, gff_type: GffType) {
    for path in paths {
        let mut reader = bio::io::gff::Reader::from_file(path, gff_type).unwrap();
        for record in reader.records() {
            let record = record.unwrap();
            // println!("Seqid: {}", record.seqid());
            println!("Source: {}", record.source());
            println!("Type: {}", record.feature_type());
            println!("Start: {}", record.start());
            println!("End: {}", record.end());
            // println!("Score: {}", record.score());
            // println!("Strand: {}", record.strand());
            // println!("Phase: {}", record.phase());
            println!("Attributes: {:?}", record.attributes());
        }
    }
    
}
