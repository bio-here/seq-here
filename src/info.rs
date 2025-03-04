use crate::utils;
use bio::bio_types::strand::Strand;
use bio::io::fasta;
use bio::io::gff::GffType;
use colored::Colorize;
use comfy_table::presets::NOTHING;
use comfy_table::{ContentArrangement, Table};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Define the trait for the different file types
///
/// 3 ways to output the information:
/// `by file` to output to a text file,
/// `by println` to output to the terminal,
/// `by csv` to output to a csv file.
pub trait InfoOutput {
    fn by_file(paths: Vec<PathBuf>);
    fn by_println(paths: Vec<PathBuf>);
    fn by_csv(paths: Vec<PathBuf>);
}

pub struct InfoFa;
impl InfoOutput for InfoFa {
    fn by_file(paths: Vec<PathBuf>) {
        let c = info_fa(paths);
        let path = PathBuf::from("info_fa.txt");
        write_file(path, &*c);
    }

    fn by_println(paths: Vec<PathBuf>) {
        println!("{}", format_table(info_fa(paths.clone())));
    }

    fn by_csv(paths: Vec<PathBuf>) {
        let c = info_fa(paths);
        let path = PathBuf::from("info_fa.csv");
        write_file(path, &c);
    }
}

pub struct InfoFq;
impl InfoOutput for InfoFq {
    fn by_file(paths: Vec<PathBuf>) {
        let c = info_fq(paths);
        let path = PathBuf::from("info_fq.txt");
        write_file(path, &c);
    }

    fn by_println(paths: Vec<PathBuf>) {
        println!("{}", info_fq(paths));
    }

    fn by_csv(paths: Vec<PathBuf>) {
        let c = info_fq(paths);
        let path = PathBuf::from("info_fq.csv");
        write_file(path, &c);
    }
}

pub struct InfoGff;
impl InfoOutput for InfoGff {
    fn by_file(paths: Vec<PathBuf>) {
        let c = info_gff(paths, GffType::GFF3);
        let path = PathBuf::from("info_gff.txt");
        write_file(path, &c);
    }

    fn by_println(paths: Vec<PathBuf>) {
        println!("{}", info_gff(paths, GffType::GFF3));
    }

    fn by_csv(paths: Vec<PathBuf>) {
        let c = info_gff(paths, GffType::GFF3);
        let path = PathBuf::from("info_gff.txt");
        write_file(path, &c);
    }
}

fn info_fa(paths: Vec<PathBuf>) -> String {
    let mut str_buf: Vec<String> = Vec::new();

    for (i, path) in paths.iter().enumerate() {
        let reader = fasta::Reader::from_file(&path)
            .expect(format!("{} reading file {}.", "Error".red().bold(), &path.display()).as_str());
        str_buf.push(format!("File: {:?} \n", path));
        str_buf.push(format!(
            "{}\t{}\t{}\t{}\t{}\t\n",
            "ID", "Seq Type", "Description", "Length", "GC content"
        ));
        let (mut count, mut total_len) = (0, 0);

        for record in reader.records() {
            let record =
                record.expect(format!("{} reading record.", "Error".red().bold()).as_str());
            let s_type = utils::try_seq_type_seq(record.seq());
            str_buf.push(format!(
                "{}\t{}\t{}\t{}\t{:.2}\t\n",
                record.id(),
                s_type,
                record.desc().unwrap_or("None"),
                record.seq().len(),
                match s_type.as_str() {
                    "DNA" => bio::seq_analysis::gc::gc_content(&*record.seq()),
                    _ => 0.0,
                }
            ));

            total_len += record.seq().len();
            count += 1;
        }
        str_buf.insert(
            i,
            format!(
                "File'{}' Total length/count : {}/{} \n",
                path.display(),
                total_len,
                count
            ),
        );
    }
    str_buf.push("\n".to_string());
    str_buf.into_iter().collect::<String>()
}

fn info_fq(paths: Vec<PathBuf>) -> String {
    let mut str_buf: Vec<String> = Vec::new();

    for (i, path) in paths.iter().enumerate() {
        let reader = bio::io::fastq::Reader::from_file(&path)
            .expect(format!("{} reading file {}.", "Error".red().bold(), &path.display()).as_str());
        str_buf.push(format!("File: {:?} \n", path));
        str_buf.push(format!(
            "{}\t{}\t{}\t{}\t\n",
            "ID", "Description", "Length", "Quality"
        ));
        let (mut count, mut total_len) = (0, 0);

        for record in reader.records() {
            let record =
                record.expect(format!("{} reading record.", "Error".red().bold()).as_str());
            str_buf.push(format!(
                "{}\t{}\t{}\t{}\t\n",
                record.id(),
                record.desc().unwrap_or("None"),
                record.seq().len(),
                record.qual().len()
            ));

            total_len += record.seq().len();
            count += 1;
        }
        str_buf.insert(
            i,
            format!(
                "File'{}' Total length/count : {}/{} \n",
                path.display(),
                total_len,
                count
            ),
        );
    }

    str_buf.push("\n".to_string());
    str_buf.into_iter().collect::<String>()
}

fn info_gff(paths: Vec<PathBuf>, gff_type: GffType) -> String {
    let mut str_buf: Vec<String> = Vec::new();

    for (i, path) in paths.iter().enumerate() {
        let mut reader = bio::io::gff::Reader::from_file(&path, gff_type)
            .expect(format!("{} reading file {}.", "Error".red().bold(), &path.display()).as_str());
        str_buf.push(format!("File: {:?} \n", path));

        let mut count = 0;
        let (mut seq_id, mut source, mut feature_type, mut score, mut strand) = (
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
            HashMap::new(),
        );

        for record in reader.records() {
            let record = record.expect("Error reading record.");

            *seq_id.entry(record.seqname().to_owned()).or_insert(0) += 1;
            *source.entry(record.source().to_owned()).or_insert(0) += 1;
            *feature_type
                .entry(record.feature_type().to_owned())
                .or_insert(0) += 1;
            *score.entry(record.score().to_owned()).or_insert(0) += 1;
            *strand
                .entry(match record.strand().unwrap_or(Strand::Unknown) {
                    Strand::Forward => "+",
                    Strand::Reverse => "-",
                    Strand::Unknown => ".",
                })
                .or_insert(0) += 1;

            count += 1;
        }

        str_buf.push(format!(
            "Seq ID: \n  {:?}\nSource: \n  {:?}\nFeature Type: \n  {:?}\nScore: \n  {:?}\nStrand: \n  {:?}\nCount: \n  {}\n",
            seq_id, source, feature_type, score, strand, count
        ));

        str_buf.insert(
            i,
            format!("File'{}' Total count : {} \n", path.display(), count),
        );
    }

    str_buf.push("\n".to_string());
    str_buf.into_iter().collect::<String>()
}

fn format_table(input: String) -> String {
    let rows: Vec<Vec<&str>> = input
        .split('\n')
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.split('\t').collect())
        .collect();

    let mut table = Table::new();
    table
        .load_preset(NOTHING)
        .set_content_arrangement(ContentArrangement::Dynamic);

    if let Some(headers) = rows.first() {
        table.set_header(headers);
    }
    for row in rows.iter().skip(1) {
        table.add_row(row);
    }

    table.to_string()
}

fn write_file<P: AsRef<Path>>(path: P, content: &str) {
    fs::write(path, content).expect("Unable to write file");
}
