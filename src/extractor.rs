use anyhow::{Result, Context};
use std::path::{Path, PathBuf};

pub struct Extractor {
    env_dir: PathBuf,
}

impl Extractor {
    pub fn new(env_dir: PathBuf) -> Self {
        Self { env_dir }
    }
    
    pub async fn extract_packages(&self, cache_files: &[PathBuf]) -> Result<()> {
        println!("\n📦 Extracting {} packages...", cache_files.len());
        
        // Create environment directories
        tokio::fs::create_dir_all(&self.env_dir).await?;
        tokio::fs::create_dir_all(self.env_dir.join("bin")).await?;
        tokio::fs::create_dir_all(self.env_dir.join("lib")).await?;
        tokio::fs::create_dir_all(self.env_dir.join("include")).await?;
        
        for (i, cache_file) in cache_files.iter().enumerate() {
            self.extract_single(i + 1, cache_files.len(), cache_file).await?;
        }
        
        println!("✅ All packages extracted!");
        Ok(())
    }
    
    async fn extract_single(&self, current: usize, total: usize, cache_file: &Path) -> Result<()> {
        println!("  [{}/{}] Extracting {}...", current, total, 
                cache_file.file_name().unwrap().to_str().unwrap());
        
        // Use extract_conda for .conda files (not extract)
        rattler_package_streaming::tokio::fs::extract_tar_bz2(cache_file, &self.env_dir)
            .await
            .context("Failed to extract package")?;
        
        Ok(())
    }
}