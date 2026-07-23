// git-sheets: Version control for spreadsheets - CLI entry point
use clap::Parser;

mod cli;

fn main() {
    let cli = cli::Cli::parse();
    
    if let Err(e) = cli.execute() {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
