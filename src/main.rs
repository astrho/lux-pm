use clap::{Parser, Subcommand};
mod manifest;
use manifest::Manifest;
use std::path::PathBuf;
use std::fs;
use anyhow::Result;

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

fn get_cache_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap();
    PathBuf::from(home).join(".lux/cache/pool")
}

fn init_cache() -> Result<()> {
    let cache_dir = get_cache_dir();
    fs::create_dir_all(&cache_dir)?;
    println!("✅ Cache initialized at: {}", cache_dir.display());
    Ok(())
}

fn main() -> Result<()> {
    init_cache()?;
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Install { package } => {
            if let Ok(manifest) = Manifest::from_file("lux.toml") {
                println!("📦 Project: {} v{}", manifest.package.name, manifest.package.version);
                if let Some(deps) = manifest.dependencies {
                    println!("📋 Dependencies:");
                    for (name, version) in deps {
                        println!("  - {} = {}", name, version);
                    }
                }
            } else {
                println!("📦 Installing single package: {}", package);
            }
            println!("(Implementation: Week 2)");
        }
        Commands::List => {
            let cache_dir = get_cache_dir();
            if cache_dir.exists() {
                let entries = fs::read_dir(&cache_dir)?;
                let count = entries.count();
                println!("📋 Cache: {}", cache_dir.display());
                println!("   {} artifacts cached", count);
            } else {
                println!("📋 No cache found");
            }
        }
    }
    Ok(())
}
