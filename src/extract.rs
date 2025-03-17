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

pub struct ExtractSegment;


// TODO: .expect("")  msg
// TODO:

impl ExtractSegment {
    pub fn extract_id(paths: Vec<PathBuf>, id: String, output: PathBuf) {
        let id_set = vec![Self::normalize_id(&id)].into_iter().collect();
        Self::process_files_parallel(paths, &id_set, &output)
    }

    pub fn extract_id_files(paths: Vec<PathBuf>, id_file: PathBuf, output: PathBuf) {
        let id_set = match Self::load_id_set(&id_file) {
            Ok(set) => set,
            Err(e) => e_exit("ID-LOAD", &format!("Failed to load IDs: {}", e), 1),
        };
        Self::process_files_parallel(paths, &id_set, &output)
    }

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
            // .unwrap_or_else(|e| { e_exit("PROCESS-ERROR", &format!("Process {} failed: {}", path.display(), e), 4) } );
    }

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

    fn normalize_id(raw_id: &str) -> String {
        raw_id.split(|c: char| c.is_whitespace() || c == '|' || c == ';')
            .next()
            .unwrap_or(raw_id)
            .to_lowercase()
    }

    fn process_fasta(path: &PathBuf, ids: &HashSet<String>, writer: &mut MultiFormatWriter) {
        let reader = fasta::Reader::from_file(path).expect("");
        for record in reader.records() {
            let record = record.expect("");
            if ids.contains(&Self::normalize_id(record.id())) {
                writer.fa.write_record(&record).expect("");
            }
        }
    }

    fn process_gff(path: &PathBuf, ids: &HashSet<String>, writer: &mut MultiFormatWriter) {
        let mut reader = gff::Reader::from_file(path, GffType::GFF3).expect("");    //TODO: GFF TYPE
        for record in reader.records() {
            let record = record.expect("");
            if let Some(id) = record.attributes().get("ID") {
                if ids.contains(&Self::normalize_id(id)) {
                    writer.gff.write(&record).expect("");
                }
            }
        }
    }

    fn process_fastq(path: &PathBuf, ids: &HashSet<String>, writer: &mut MultiFormatWriter) {
        let reader = fastq::Reader::from_file(path).expect("");
        for record in reader.records() {
            let record = record.expect("");
            if ids.contains(&Self::normalize_id(record.id())) {
                writer.fq.write_record(&record).expect("");
            }
        }
    }
}



pub struct ExtractExplain;

impl ExtractExplain {
    pub fn extract(seq_files: Vec<PathBuf>, anno_files: Vec<PathBuf>, output: PathBuf) {
        // 创建输出目录
        fs::create_dir_all(&output).unwrap_or_else(|e| {
            e_exit("FS", &format!("创建输出目录失败: {}", e), 1);
        });

        // 并行处理每个序列文件
        seq_files.par_iter().for_each(|seq_path| {
            // 加载序列数据
            let seq_data = Self::load_sequences(seq_path)
                .unwrap_or_else(|e| e_exit("SEQ-LOAD", &e, 2));

            // 处理所有注释文件
            let annotations: Vec<_> = anno_files.par_iter()
                .flat_map(|anno_path| {
                    Self::load_annotations(anno_path)
                        .unwrap_or_else(|e| e_exit("ANN-LOAD", &e, 3))
                })
                .collect();

            // 生成注释结果
            let output_path = output.join(seq_path.file_name().unwrap());
            Self::generate_annotated_file(&seq_data, &annotations, &output_path)
                .unwrap_or_else(|e| e_exit("OUTPUT", &e, 4));
        });
    }

    /// 加载序列数据到内存（适合中小型文件）
    fn load_sequences(path: &Path) -> Result<HashMap<String, fasta::Record>, String> {
        let reader = fasta::Reader::from_file(path)
            .map_err(|e| format!("读取序列文件失败: {} - {}", path.display(), e))?;

        let mut seq_map = HashMap::new();
        for record in reader.records() {
            let record = record.map_err(|e| format!("解析FASTA失败: {}", e))?;
            seq_map.insert(record.id().to_string(), record);
        }
        Ok(seq_map)
    }

    /// 加载GFF注释信息
    fn load_annotations(path: &Path) -> Result<Vec<gff::Record>, String> {
        let mut reader = gff::Reader::from_file(path, GffType::GFF3)
            .map_err(|e| format!("读取注释文件失败: {} - {}", path.display(), e))?;

        reader.records()
            .map(|r| r.map_err(|e| format!("解析GFF失败: {}", e)))
            .collect()
    }

    /// 生成带注释的序列文件
    fn generate_annotated_file(
        seq_data: &HashMap<String, fasta::Record>,
        annotations: &[gff::Record],
        output: &Path
    ) -> Result<(), String> {
        let mut writer = fasta::Writer::new(File::create(output)
            .map_err(|e| format!("创建输出文件失败: {} - {}", output.display(), e))?);

        // 为每个注释生成特征序列
        for ann in annotations {
            let seq_id = ann.seqname();
            let Some(seq) = seq_data.get(seq_id) else {
                e_println("ANN-SKIP", &format!("未找到序列: {}", seq_id));
                continue;
            };

            // 提取注释区间序列
            let feature_seq = Self::extract_feature(seq, ann)
                .map_err(|e| format!("提取特征失败: {}", e))?;

            // 生成描述信息
            let description = format!("{}:{}-{} {}",
                                      ann.feature_type(),
                                      ann.start(),
                                      ann.end(),
                                      ann.attributes().get("ID").unwrap_or(&"unknown".to_string())
            );

            // 写入新记录
            let new_record = fasta::Record::with_attrs(
                ann.attributes().get("ID").unwrap_or(&ann.seqname().to_string()),
                Some(&description),
                &feature_seq
            );
            writer.write_record(&new_record)
                .map_err(|e| format!("写入失败: {}", e))?;
        }
        Ok(())
    }

    /// 从序列中提取特征区间
    fn extract_feature(seq: &fasta::Record, ann: &gff::Record) -> Result<Vec<u8>, String> {
        let start = ann.start().saturating_sub(1); // GFF是1-based
        let &end = ann.end();

        if start >= seq.seq().len() as u64 || end > seq.seq().len() as u64 {
            return Err(format!("无效区间: {}-{} (序列长度: {})",
                               ann.start(), ann.end(), seq.seq().len()));
        }

        Ok(seq.seq()[start as usize..end as usize].to_vec())
    }
}

