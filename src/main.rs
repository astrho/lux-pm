mod manifest;
mod resolver;

use manifest::Manifest;
use resolver::Resolver;
use clap::{Parser, Subcommand};
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
    /// Install packages from lux.toml or by name
    Install { 
        #[arg(default_value = "")]
        package: String 
    },
    
    /// List installed packages and cache stats
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

#[tokio::main]
async fn main() -> Result<()> {
    init_cache()?;
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Install { package } => {
            // Get dependencies
            let specs = if package.is_empty() {
                // Read from lux.toml
                let manifest = Manifest::from_file("lux.toml")?;
                println!("📦 Project: {} v{}", manifest.package.name, manifest.package.version);
                
                if let Some(deps) = manifest.dependencies {
                    deps.into_iter()
                        .map(|(name, version)| format!("{} {}", name, version))
                        .collect()
                } else {
                    println!("⚠️  No dependencies found in lux.toml");
                    return Ok(());
                }
            } else {
                // Single package from CLI
                vec![package]
            };
            
            // Resolve dependencies
            let resolver = Resolver::new().await?;
            let solution = resolver.solve(&specs)?;
            
            println!("\n📦 Ready to install {} packages", solution.len());
            println!("(Download step: Wednesday)");
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