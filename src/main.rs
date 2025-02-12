use clap::{ArgMatches, Args, Command, Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "seq-here", next_line_help = true)]
#[command(author = "Zhixia Lau <zhixiaovo@gmail.com>")]
#[command(version = "1.0", about = "A fast tool for bio-sequence file processing", long_about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    name: Option<String>
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
    #[command(about = "Extracts specified sequence information or file data.")]
    Extract(ExtractCmd),
}

#[derive(Subcommand)]
enum InfoCmd {
    #[command(about = "Fasta file information.")]
    Fa(InfoFaArgs),

    #[command(about = "Fastq file information.")]
    Fq(InfoFqArgs),

    #[command(about = "Gff/Gtf file information. Gff2 not supported yet due to upstream rust-bio.")]
    Gff(InfoGffArgs),
}

#[derive(Args)]
struct InfoFaArgs {
    #[command(flatten)]
    input: InputFile,

    #[arg(short, long, required = true, value_enum)]
    #[arg(help = "Info mode")]
    mode: InfoFaArgsMode,
}

#[derive(Copy, Clone, ValueEnum)]
enum InfoFaArgsMode {
    #[value(help = "Prints general information about the file.")]
    Status,

    #[value(help = "Output information about all sequences.")]
    Each,

    #[value(help = "Output all the information the options above would generate")]
    All,
}

#[derive(Args)]
struct InfoFqArgs {
    #[command(flatten)]
    input: InputFile,

}

#[derive(Args)]
struct InfoGffArgs {
    #[arg(long)]
    input: String,

}

#[derive(Subcommand)]
enum ConvertCmd {

}

#[derive(Subcommand)]
enum ExtractCmd {

}


#[derive(Args)]
struct InputFile {
    #[arg(short = 'f', long)]
    #[arg(help = "Input files or the directory containing the files.")]
    #[arg(value_name = "FILES")]
    files: Vec<String>,
}

impl InputFile {

}


fn main() {
    let args = Cli::parse();

    match args.command {

        Commands::Info(info_cmd) => {
            match info_cmd {
                InfoCmd::Fa(args) => {
                    println!("Input1: {}", args.input.files.first().unwrap());
                    match args.mode {
                        InfoFaArgsMode::Status => {
                            
                        }
                        InfoFaArgsMode::Each => {
                            
                        }
                        InfoFaArgsMode::All => {
                            
                        }
                    }
                },
                
                InfoCmd::Fq(args) => {
                    println!("Input: {}", args.input.files.first().unwrap());
                },
                
                InfoCmd::Gff(args) => {
                    println!("Input: {}", args.input);
                }
            }
            println!("Info command");
        },

        Commands::Convert(_) => {
            println!("Convert command");
        },

        Commands::Extract(_) => {
            println!("Extract command");
        }

    }

}
