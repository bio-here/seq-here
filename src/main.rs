use clap::{Args, Parser, Subcommand, ValueEnum};
use colored::Colorize;
use seq_here::error::e_exit;
use seq_here::extract::{self};
use seq_here::info::{self, InfoOutput};
use seq_here::process::{self};
use seq_here::utils;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser)]
#[command(name = "seq-here", next_line_help = true)]
#[command(author = "Zhixia Lau <zhixiaovo@gmail.com>")]
#[command(
    version = "0.0.4",
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
    #[arg(long, short = 'o', default_value = "println")] // default = "println"
    output_type: OutputType,
}

#[derive(Args)]
struct InfoFqArgs {
    #[command(flatten)]
    input: InputFile,

    #[arg(long, short = 'o', default_value = "println")] // default = "println"
    output_type: OutputType,
}

#[derive(Args)]
struct InfoGffArgs {
    #[command(flatten)]
    input: InputFile,

    #[arg(long, short = 't', default_value = "gff3")] // default = "gff3"
    _type: Option<String>,

    #[arg(long, short = 'o', default_value = "println")] // default = "println"
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
    #[command(about = "Combine the given files into one file, support all-type text files.(TODO)")] //TODO: Combine
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
    input: InputFile,           // The files that store the sequence to be extracted

    #[command(flatten)]
    id_options: InputOptions,   // Input ids by keyboard or file

    #[command(flatten)]
    output: OutputFile,         // Output file
}

#[derive(Args)]
struct ExtractExplainArgs {
    #[command(flatten)]
    input1: InputFile,          // Sequence files

    #[command(flatten)]
    input2: InputFile,          // GFF files

    #[command(flatten)]
    output: OutputFile,         // Output file
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
    // #[arg(short = 'f', long)]
    #[arg(required = true)]
    #[arg(help = "Input files or the directory containing the files, seperated by ',' .")]
    #[arg(value_name = "FILES")]
    #[arg(value_delimiter = ',')]
    files: Vec<PathBuf>,
}

impl InputFile {
    // if the input is a file, return the file path
    // if the input is a directory, return all the files in the directory
    fn get_files(&self) -> Vec<PathBuf> {
        let mut files = Vec::new();
        for f in &self.files {
            if !f.exists() {
                e_exit("File", "File(s) does not exist.", 1);
            }

            let f = Path::new(f);
            if f.is_dir() {
                for e in f.read_dir().unwrap() {
                    let e = e.unwrap();
                    let path = e.path();
                    if path.is_file() {
                        files.push(path.to_path_buf());
                    }
                }
            }
            if f.is_file() {
                files.push(f.to_path_buf());
            }
        }

        files
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
    // Check if path does exist.
    // Return a file path.
    fn get_file(&self, default: &str) -> PathBuf {
        match &self.output {
            Some(path) => {
                if let Ok(metadata) = fs::metadata(path) {
                    if metadata.is_file() {
                        path.clone()
                    } else {
                        let new_path = path.join(default);
                        // utils::create_file_with_dir(&new_path);
                        new_path
                    }
                } else {
                    if utils::is_directory_path(path) {
                        let dir_path = path.join(default);
                        // utils::create_file_with_dir(&dir_path);
                        dir_path
                    } else {
                        // utils::create_file_with_dir(path);
                        path.clone()
                    }
                }
            }
            None => {
                let default_path = PathBuf::from("./").join(default);
                // utils::create_file_with_dir(&default_path);
                default_path
            }
        }
    }

}


fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Info(info_cmd) => match info_cmd {
            InfoCmd::Fa(args) => {
                let files = args.input.get_files();
                println!("{}: {:?}", "Inputs:".green().bold(), files);
                match args.output_type {
                    OutputType::File => {
                        info::InfoFa::by_file(files, vec![]);
                    }
                    OutputType::Println => {
                        info::InfoFa::by_println(files, vec![]);
                    }
                    OutputType::Csv => {
                        info::InfoFa::by_csv(files, vec![]);
                    }
                }
            }

            InfoCmd::Fq(args) => {
                let files = args.input.get_files();
                println!("{}: {:?}", "Inputs:".green().bold(), files);
                match args.output_type {
                    OutputType::File => {
                        info::InfoFq::by_file(files, vec![]);
                    }
                    OutputType::Println => {
                        info::InfoFq::by_println(files, vec![]);
                    }
                    OutputType::Csv => {
                        info::InfoFq::by_csv(files, vec![]);
                    }
                }
            }

            InfoCmd::Gff(args) => {
                let files = args.input.get_files();
                println!("{}: {:?}", "Inputs:".green().bold(), files);
                match args.output_type {
                    OutputType::File => {
                        info::InfoGff::by_file(files, vec!["gff3".to_string()]);
                    }
                    OutputType::Println => {
                        info::InfoGff::by_println(files, vec!["gff3".to_string()]);
                    }
                    OutputType::Csv => {
                        info::InfoGff::by_csv(files, vec!["gff3".to_string()]);
                    }
                }
            }
        },

        Commands::Process(process_cmd) => match process_cmd{
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

                match (args.id_options.file, args.id_options.str) {
                    (None, Some(id)) => {
                        println!("{}: {:?}", "Input ID:".yellow().bold(), id);
                        extract::ExtractSegment::extract_id(seq_files, id, out);
                    },
                    (Some(path), None) => {
                        println!("{}: {:?}", "Input path:".yellow().bold(), path);
                        extract::ExtractSegment::extract_id_files(seq_files, path, out);
                    },
                    _ => {}
                };
            },

            ExtractCmd::Explain(args) => {
                let (seq_files, gff_files) = (args.input1.get_files(), args.input2.get_files());
                let out = args.output.get_file("./anno_extracted_segment");
                println!("{}: {:?}\n{}: {:?}",
                         "Input sequence files:".green().bold(), seq_files,
                         "Input annotation files".yellow().bold(), gff_files);
                extract::ExtractExplain::extract(seq_files, gff_files, out);
            }
        },
    }
}
