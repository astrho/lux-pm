use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "lux", version, about = "Lux Package Manager")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Install { package: String },
    List,
}

fn main() {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Install { package } => {
            println!("📦 Installing {}", package);
            println!("(Implementation: Week 2)");
        }
        Commands::List => {
            println!("📋 No packages installed yet");
        }
    }
}
