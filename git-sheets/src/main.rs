// git-sheets: Version control for spreadsheets - CLI entry point
use clap::Parser;
use gitsheets::cli::Cli;

fn main() {
    let cli = Cli::parse();

    if let Err(e) = cli.execute() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
