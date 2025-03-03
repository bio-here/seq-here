use bio::io::gff::GffType;
use clap::{Args, Parser, Subcommand};
use colored::Colorize;
use seq_here::info::{self, info_gff, InfoOutput};
use std::path::{Path, PathBuf};

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

#[derive(Subcommand)]
enum Commands {
    #[command(subcommand)]
    #[command(about = "Get basic information about the input sequence file(s).")]
    Info(InfoCmd),

    #[command(subcommand)]
    #[command(about = "Convert or process incoming sequence file(s).")]
    Convert(ConvertCmd),

    #[command(subcommand)]
    #[command(about = "Extract specified sequence segment or file data.")]
    Extract(ExtractCmd),
}

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
}

#[derive(Args)]
struct InfoFqArgs {
    #[command(flatten)]
    input: InputFile,
}

#[derive(Args)]
struct InfoGffArgs {
    #[command(flatten)]
    input: InputFile,

    #[arg(long, short = 't',
        default_value = "gff3")] // default = "gff3"
    _type: Option<String>,
}

#[derive(Subcommand)]
enum ConvertCmd {

}

#[derive(Subcommand)]
enum ExtractCmd {

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
    fn get_files(&self) -> Vec<PathBuf> {

        let mut files = Vec::new();
        for f in &self.files {
            if !f.exists() {
                eprintln!("{}: File not found: {}", "Error".red().bold(), f.to_str().unwrap());
                std::process::exit(1);
            }

            let f = Path::new(f);
            if f.is_dir() {
                for e in f.read_dir().unwrap() {
                    let e = e.unwrap();
                    let path = e.path();
                    if path.is_file() {
                        files.push(f.to_path_buf());
                    }
                }
                eprintln!("Directory provided: {}", f.to_str().unwrap());
                std::process::exit(1);
            }
            if f.is_file() {
                files.push(f.to_path_buf());
            }
        }

        files
    }
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Info(info_cmd) => {
            match info_cmd {
                InfoCmd::Fa(args) => {
                    println!(
                        "{}: {:?}",
                        "Inputs:".green().bold(),
                        args.input.get_files()
                    );
                    info::InfoFa::by_file(args.input.get_files());
                    info::InfoFa::by_println(args.input.get_files());
                }

                InfoCmd::Fq(args) => {
                    println!(
                        "{}: {:?}",
                        "Inputs:".green().bold(),
                        args.input.get_files()
                    );
                    info::InfoFq::by_file(args.input.get_files());
                    info::InfoFq::by_println(args.input.get_files());
                }

                InfoCmd::Gff(args) => {
                    println!(
                        "{}: {:?}",
                        "Inputs:".green().bold(),
                        args.input.get_files()
                    );
                    info_gff(args.input.get_files(), GffType::GFF3);
                    info::InfoGff::by_file(args.input.get_files());
                    info::InfoGff::by_println(args.input.get_files());
                }
            }

        }

        Commands::Convert(_) => {
            println!("Convert command");
            println!("{}", "Not implemented yet".yellow().bold());

        }

        Commands::Extract(_) => {
            println!("Extract command");
            println!("{}", "Not implemented yet".yellow().bold());
        }
    }
}
