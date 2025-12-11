use rattler_conda_types::{MatchSpec, RepoData, RepoDataRecord, ParseStrictness};
use rattler_solve::{resolvo::Solver, SolverImpl, SolverTask, ChannelPriority};
use anyhow::{Result, Context};
use url::Url;

pub struct Resolver {
    records: Vec<RepoDataRecord>,
}

impl Resolver {
    pub async fn new() -> Result<Self> {
        println!("📥 Fetching package index from conda-forge...");
        
        let platform = if cfg!(target_arch = "aarch64") && cfg!(target_os = "macos") {
            "osx-arm64"
        } else if cfg!(target_arch = "x86_64") && cfg!(target_os = "macos") {
            "osx-64"
        } else if cfg!(target_arch = "aarch64") && cfg!(target_os = "linux") {
            "linux-aarch64"
        } else {
            "linux-64"
        };
        
        let url = format!(
            "https://conda.anaconda.org/conda-forge/{}/repodata.json",
            platform
        );
        
        println!("   Platform: {}", platform);
        println!("   URL: {}", url);
        
        let response = reqwest::get(&url)
            .await
            .context("Failed to download repodata")?;
        
        let bytes = response.bytes()
            .await
            .context("Failed to read repodata bytes")?;
        
        println!("   Downloaded {} MB", bytes.len() / 1_000_000);
        
        let repodata: RepoData = serde_json::from_slice(&bytes)
            .context("Failed to parse repodata JSON")?;
        
        // Convert to Vec<RepoDataRecord>
        let records: Vec<RepoDataRecord> = repodata
            .packages
            .into_iter()
            .map(|(file_name, record)| RepoDataRecord {
                url: Url::parse(&format!(
                    "https://conda.anaconda.org/conda-forge/{}/{}",
                    platform, file_name
                )).unwrap(),
                channel: Some("conda-forge".to_string()),
                file_name,
                package_record: record,
            })
            .collect();
        
        println!("✅ Loaded {} packages", records.len());
        
        Ok(Self { records })
    }
    
    pub fn solve(&self, specs: &[String]) -> Result<Vec<RepoDataRecord>> {
        println!("🔍 Resolving dependencies...");
        
        let match_specs: Vec<MatchSpec> = specs
            .iter()
            .map(|s| MatchSpec::from_str(s, ParseStrictness::Lenient))
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to parse package specs")?;
        
        println!("   Requirements:");
        for spec in &match_specs {
            println!("     - {}", spec);
        }
        
        // Create solver task with ALL fields
        let task = SolverTask {
            specs: match_specs,
            available_packages: vec![&self.records],
            locked_packages: vec![],
            pinned_packages: vec![],
            virtual_packages: vec![],
            channel_priority: ChannelPriority::Strict,
            constraints: vec![],
            exclude_newer: None,
            min_age: None,
            strategy: Default::default(),
            timeout: None,
        };
        
        // Solve!
        let mut solver = Solver;
        let solution = solver
            .solve(task)
            .context("Failed to solve dependencies")?;
        
        println!("✅ Resolved {} packages:", solution.records.len());
        for record in &solution.records {
            println!("   - {} {}", 
                record.package_record.name.as_normalized(), 
                record.package_record.version
            );
        }
        
        Ok(solution.records)
    }
}
