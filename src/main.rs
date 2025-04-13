use clap::{Args, Parser, Subcommand, ValueEnum};
use colored::Colorize;
use seq_here::error::e_exit;
use seq_here::extract::{self};
use seq_here::info::{self, InfoOutput};
use seq_here::process::{self};
use seq_here::utils;
use std::fs;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "seq-here", next_line_help = true)]
#[command(author = "Zhixia Lau <zhixiaovo@gmail.com>")]
#[command(
    version = "0.1.0",
    about = "A fast tool for bio-sequence file processing",
    long_about
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

/// Commands List
///
#[derive(Subcommand)]
enum Commands {
    #[command(subcommand)]
    #[command(about = "Get basic information about the input sequence file(s).")]
    Info(InfoCmd),

    #[command(subcommand)]
    #[command(about = "Convert or process incoming sequence file(s).")]
    Process(ProcessCmd),

    #[command(subcommand)]
    #[command(about = "Extract specified sequence segments.")]
    Extract(ExtractCmd),
}

/// Info Subcommand
///
#[derive(Subcommand)]
enum InfoCmd {
    #[command(about = "Fasta file information.")]
    Fa(InfoFaArgs),

    #[command(about = "Fastq file information.")]
    Fq(InfoFqArgs),

    #[command(
        about = "Gff/Gtf file information. Gff2 not supported yet due to upstream bio crate."
    )]
    Gff(InfoGffArgs),
}

#[derive(Args)]
struct InfoFaArgs {
    #[command(flatten)]
    input: InputFile,
    #[arg(value_enum)]
    #[arg(long, short = 'o', default_value = "println")]
    output_type: OutputType,
}

#[derive(Args)]
struct InfoFqArgs {
    #[command(flatten)]
    input: InputFile,

    #[arg(long, short = 'o', default_value = "println")]
    output_type: OutputType,
}

#[derive(Args)]
struct InfoGffArgs {
    #[command(flatten)]
    input: InputFile,

    #[arg(long, short = 't', default_value = "gff3")]
    _type: Option<String>,

    #[arg(long, short = 'o', default_value = "println")]
    output_type: OutputType,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputType {
    File,
    Println,
    Csv,
}

/// Process Subcommand
///
#[derive(Subcommand)]
enum ProcessCmd {
    #[command(about = "Combine the given files into one file, support all-type text files.(TODO)")]
    Combine(ProcessCombineArgs),
}

#[derive(Args)]
struct ProcessCombineArgs {
    #[command(flatten)]
    input: InputFile,

    #[command(flatten)]
    output: OutputFile,
}

/// Extract Subcommand
///
#[derive(Subcommand)]
enum ExtractCmd {
    #[command(about = "Extract sequence segments by given id(s) or pattern(s).")]
    Segment(ExtractSegmentArgs),

    #[command(about = "Extract sequence segment from fasta file by given gff file.")]
    Explain(ExtractExplainArgs),
}

#[derive(Args)]
struct ExtractSegmentArgs {
    #[command(flatten)]
    input: InputFile,

    #[command(flatten)]
    id_options: InputOptions,

    #[arg(short, long)]
    #[arg(help = "Optional start position (0-based) for the extracted segment")]
    start: Option<usize>,

    #[arg(short, long)]
    #[arg(help = "Optional end position (0-based, exclusive) for the extracted segment")]
    end: Option<usize>,

    #[command(flatten)]
    output: OutputFile,
}

#[derive(Args)]
struct ExtractExplainArgs {
    #[arg(short = 's', long = "seq")]
    #[arg(required = true)]
    #[arg(help = "Input sequence files (FASTA), separated by ',' .")]
    #[arg(value_name = "SEQ_FILES")]
    #[arg(value_delimiter = ',')]
    seq_files: Vec<PathBuf>,

    #[arg(short = 'g', long = "gff")]
    #[arg(required = true)]
    #[arg(help = "Input annotation files (GFF/GTF), separated by ',' .")]
    #[arg(value_name = "GFF_FILES")]
    #[arg(value_delimiter = ',')]
    gff_files: Vec<PathBuf>,

    #[arg(short = 't', long = "type")]
    #[arg(help = "Feature types to extract (e.g., 'CDS,gene,mRNA'), separated by ',' .")]
    #[arg(value_name = "FEATURE_TYPES")]
    #[arg(value_delimiter = ',')]
    feature_types: Option<Vec<String>>,

    #[command(flatten)]
    output: OutputFile,
}

/// I/O Options
///
#[derive(Args)]
#[group(required = true, multiple = false)]
struct InputOptions {
    #[arg(short = 's', long)]
    #[arg(help = "Directly input text")]
    #[arg(value_name = "String")]
    str: Option<String>,

    #[arg(short = 'f', long)]
    #[arg(help = "Input path to file containing text(one per line)")]
    #[arg(value_name = "File Path")]
    file: Option<PathBuf>,
}

#[derive(Args)]
struct InputFile {
    #[arg(required = true)]
    #[arg(help = "Input files or the directory containing the files, seperated by ',' .")]
    #[arg(value_name = "FILES")]
    #[arg(value_delimiter = ',')]
    files: Vec<PathBuf>,
}

impl InputFile {
    fn get_files(&self) -> Vec<PathBuf> {
        expand_file_paths(&self.files)
    }
}

#[derive(Args)]
struct OutputFile {
    #[arg(short = 'o', long)]
    #[arg(help = "Output file name, if value is a directory, \
     it would use default file_name in the directory.")]
    #[arg(value_name = "OutputFile")]
    output: Option<PathBuf>,
}

impl OutputFile {
    fn get_file(&self, default: &str) -> PathBuf {
        match &self.output {
            Some(path) => {
                if let Ok(metadata) = fs::metadata(path) {
                    if metadata.is_file() {
                        return path.clone();
                    }
                    // 是目录，在目录中创建默认文件
                    return path.join(default);
                }
                
                // 路径不存在
                if utils::is_directory_path(path) {
                    path.join(default)
                } else {
                    path.clone()
                }
            }
            None => PathBuf::from("./").join(default),
        }
    }
}

// 通用函数：展开文件路径（处理目录和文件）
fn expand_file_paths(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for path in paths {
        if !path.exists() {
            e_exit("File", "File(s) does not exist.", 1);
        }

        if path.is_dir() {
            match path.read_dir() {
                Ok(entries) => {
                    for entry in entries.filter_map(Result::ok) {
                        let entry_path = entry.path();
                        if entry_path.is_file() {
                            files.push(entry_path);
                        }
                    }
                }
                Err(_) => e_exit("File", &format!("Cannot read directory: {:?}", path), 1),
            }
        } else if path.is_file() {
            files.push(path.clone());
        }
    }
    files
}

// 处理信息输出的通用函数
fn handle_info_output<T: InfoOutput>(files: Vec<PathBuf>, output_type: OutputType, extra_args: Vec<String>) {
    println!("{}: {:?}", "Inputs:".green().bold(), files);
    match output_type {
        OutputType::File => T::by_file(files, extra_args),
        OutputType::Println => T::by_println(files, extra_args),
        OutputType::Csv => T::by_csv(files, extra_args),
    }
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Info(info_cmd) => match info_cmd {
            InfoCmd::Fa(args) => {
                let files = args.input.get_files();
                handle_info_output::<info::InfoFa>(files, args.output_type, vec![]);
            }

            InfoCmd::Fq(args) => {
                let files = args.input.get_files();
                handle_info_output::<info::InfoFq>(files, args.output_type, vec![]);
            }

            InfoCmd::Gff(args) => {
                let files = args.input.get_files();
                handle_info_output::<info::InfoGff>(files, args.output_type, vec!["gff3".to_string()]);
            }
        },

        Commands::Process(process_cmd) => match process_cmd {
            ProcessCmd::Combine(args) => {
                let files = args.input.get_files();
                let out = args.output.get_file("./combined");
                println!("{}: {:?}", "Input files:".green().bold(), files);
                println!("{}: {:?}", "Output file:".green().bold(), out);
                process::ConvertCombine::combine_all(files, out);
            }
        },

        Commands::Extract(extract_cmd) => match extract_cmd {
            ExtractCmd::Segment(args) => {
                let seq_files = args.input.get_files();
                let out = args.output.get_file("./id_extracted_segment");
                println!("{}: {:?}", "Input files:".green().bold(), seq_files);
                
                // Display position range if specified
                if args.start.is_some() || args.end.is_some() {
                    println!("{}: {}..{}",
                             "Position range:".yellow().bold(), 
                             args.start.map_or("start".to_string(), |s| s.to_string()),
                             args.end.map_or("end".to_string(), |e| e.to_string()));
                }

                match (args.id_options.file, args.id_options.str) {
                    (None, Some(id)) => {
                        println!("{}: {:?}", "Input ID:".yellow().bold(), id);
                        extract::ExtractSegment::extract_id(seq_files, id, out, args.start, args.end);
                    },
                    (Some(path), None) => {
                        println!("{}: {:?}", "Input path:".yellow().bold(), path);
                        extract::ExtractSegment::extract_id_files(seq_files, path, out, args.start, args.end);
                    },
                    _ => {}
                };
            },

            ExtractCmd::Explain(args) => {
                let seq_files = expand_file_paths(&args.seq_files);
                let gff_files = expand_file_paths(&args.gff_files);
                let out = args.output.get_file("./anno_extracted_segment");
                
                println!("{}: {:?}\n{}: {:?}",
                         "Input sequence files:".green().bold(), seq_files,
                         "Input annotation files:".yellow().bold(), gff_files);
                
                if let Some(types) = &args.feature_types {
                    println!("{}: {:?}", "Feature types filter:".yellow().bold(), types);
                }
                
                extract::ExtractExplain::extract(seq_files, gff_files, out, args.feature_types.clone());
            }
        }
    }
}
