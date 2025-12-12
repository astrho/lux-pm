mod manifest;
mod resolver;
mod downloader;
mod extractor;

use extractor::Extractor;
use clap::{Parser, Subcommand};
use anyhow::Result;
use std::path::PathBuf;
use resolver::Resolver;
use downloader::Downloader;

#[derive(Parser)]
#[command(name = "lux")]
#[command(about = "Lux Package Manager - Fast, mesh-native robotics packages")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Install {
        package: Vec<String>,
    },
    List,
}

fn get_cache_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME")?;
    Ok(PathBuf::from(home).join(".lux/cache/pool"))
}

fn init_cache() -> Result<()> {
    let cache_dir = get_cache_dir()?;
    std::fs::create_dir_all(&cache_dir)?;
    println!("✅ Cache initialized at: {}", cache_dir.display());
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    init_cache()?;
    
    match cli.command {
        Commands::Install { package } => {
            // Determine specs
            let specs = if package.is_empty() {
                // Read from lux.toml
                let manifest = manifest::Manifest::from_file("lux.toml")?;
                println!("📋 Project: {}", manifest.package.name);
                
                if let Some(deps) = manifest.dependencies {
                    deps.into_iter()
                        .map(|(name, version)| format!("{} {}", name, version))
                        .collect()
                } else {
                    println!("No dependencies found in lux.toml");
                    return Ok(());
                }
            } else {
                package
            };
            
            // Resolve dependencies
            let resolver = Resolver::new().await?;
            let solution = resolver.solve(&specs)?;
            
            // Download packages
            let downloader = Downloader::new(get_cache_dir()?);
            downloader.download_packages(&solution).await?;

            // Extract to environment
            let env_dir = PathBuf::from(std::env::var("HOME")?)
                .join(".lux/envs/default");
            let extractor = Extractor::new(env_dir);

            let cache_files: Vec<PathBuf> = solution.iter().map(|pkg| {
                let hash = hex::encode(pkg.package_record.sha256.as_ref().unwrap());
                let mut path = get_cache_dir().unwrap()
                    .join(&hash[..2])
                    .join(&hash[2..]);
                path.set_extension("conda");  // ← ADD THIS - files need .conda extension
                path
            }).collect();

            extractor.extract_packages(&cache_files).await?;

            println!("\n🎉 Installation complete!");
        }
        
        Commands::List => {
            let cache_dir = get_cache_dir()?;
            
            if !cache_dir.exists() {
                println!("Cache is empty");
                return Ok(());
            }
            
            let mut count = 0;
            for entry in std::fs::read_dir(&cache_dir)? {
                let entry = entry?;
                if entry.path().is_dir() {
                    for sub_entry in std::fs::read_dir(entry.path())? {
                        sub_entry?;
                        count += 1;
                    }
                }
            }
            
            println!("📦 Cache: {} artifacts", count);
            println!("📍 Location: {}", cache_dir.display());
        }
    }
    
    Ok(())
}
