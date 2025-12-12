use rattler_conda_types::RepoDataRecord;
use anyhow::{Result, Context};
use sha2::{Sha256, Digest};
use std::path::PathBuf;
use tokio::fs;

pub struct Downloader {
    cache_dir: PathBuf,
}

impl Downloader {
    pub fn new(cache_dir: PathBuf) -> Self {
        Self { cache_dir }
    }
    
    pub async fn download_packages(&self, packages: &[RepoDataRecord]) -> Result<()> {
        println!("\n📥 Downloading {} packages (5 concurrent)...", packages.len());
        
        let total = packages.len();
        
        // FIX: Use semaphore for true concurrency control
        use tokio::sync::Semaphore;
        use std::sync::Arc;
        
        let semaphore = Arc::new(Semaphore::new(5)); // Max 5 concurrent
        let mut tasks = Vec::new();
        
        for (i, package) in packages.iter().enumerate() {
            let pkg = package.clone();
            let cache_dir = self.cache_dir.clone();
            let permit = semaphore.clone();
            
            let task = tokio::spawn(async move {
                let _permit = permit.acquire().await.unwrap(); // Wait for slot
                download_single_package(i + 1, total, &pkg, &cache_dir).await
            });
            
            tasks.push(task);
        }
        
        // Wait for ALL downloads to complete
        let mut errors = Vec::new();
        for task in tasks {
            if let Err(e) = task.await? {
                errors.push(e);
            }
        }
        
        if !errors.is_empty() {
            anyhow::bail!("Failed to download {} packages", errors.len());
        }
        
        println!("✅ All packages downloaded successfully!");
        Ok(())
    }
}

async fn download_single_package(
    current: usize,
    total: usize,
    package: &RepoDataRecord,
    cache_dir: &PathBuf,
) -> Result<()> {
    let name = &package.package_record.name;
    let version = &package.package_record.version;
    
    // Get expected hash from metadata
    let expected_hash_hex = hex::encode(
        package.package_record.sha256.as_ref()
            .context("Package missing SHA256")?
    );
    
    // Build cache path using METADATA hash
    let hash_dir = cache_dir.join(&expected_hash_hex[..2]);
    let file_path = hash_dir.join(format!("{}.conda", &expected_hash_hex[2..]));
    
    // Check if already cached
    if file_path.exists() {
        println!("  [{}/{}] ✓ {} {} (cached)", current, total, name.as_normalized(), version);
        return Ok(());
    }
    
    println!("  [{}/{}] {} {}...", current, total, name.as_normalized(), version);
    
    // Download file
    let response = reqwest::get(package.url.as_str())
        .await
        .context(format!("Failed to download {}", name.as_normalized()))?;
    
    let bytes = response.bytes()
        .await
        .context("Failed to read response bytes")?;
    
    // Verify SHA256
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let computed_hash = hasher.finalize();
    let computed_hash_hex = format!("{:x}", computed_hash);
    
    if computed_hash_hex != expected_hash_hex {
        anyhow::bail!(
            "Hash mismatch for {}: expected {}, got {}",
            name.as_normalized(),
            expected_hash_hex,
            computed_hash_hex
        );
    }
    
    // Save to cache using METADATA hash (not computed)
    fs::create_dir_all(&hash_dir).await?;
    let size_kb = bytes.len() / 1024; 
    fs::write(&file_path, bytes).await?;
    
    println!("    ✓ {} ({} KB)", name.as_normalized(), size_kb);
    
    Ok(())
}