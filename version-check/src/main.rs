use anyhow::{Context, Result};
use clap::Parser;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to output the versions JSON file
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Path to input the previous versions JSON file (for comparison)
    #[arg(short, long)]
    input: Option<PathBuf>,

    /// Path to output the update status JSON
    #[arg(short = 's', long)]
    update_status: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct GithubTag {
    name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct OutputVersions {
    nix_version: String,
}

#[derive(Debug, Serialize)]
struct CheckResult {
    nix_needs_update: bool,
    current: OutputVersions,
}

fn fetch_nix_tags() -> Result<Vec<GithubTag>> {
    let client = reqwest::blocking::Client::builder()
        .user_agent("version-check/0.1.0")
        .build()?;

    let url = "https://api.github.com/repos/NixOS/nix/tags";
    let resp = client.get(url).send()?.text()?;
    let tags: Vec<GithubTag> = serde_json::from_str(&resp)?;
    Ok(tags)
}

fn get_latest_stable_version(tags: &[GithubTag]) -> Result<String> {
    let mut versions = Vec::new();

    for tag in tags {
        // Strip 'v' prefix if present
        let ver_str = tag.name.trim_start_matches('v');
        if let Ok(ver) = Version::parse(ver_str) {
            // Filter out pre-releases if needed, though Nix often uses simple SemVer for stable
            if ver.pre.is_empty() {
                versions.push(ver);
            }
        }
    }

    versions.sort();

    // Get the latest version
    versions
        .last()
        .map(|v| v.to_string())
        .ok_or_else(|| anyhow::anyhow!("No valid stable versions found"))
}

fn main() -> Result<()> {
    let args = Args::parse();

    println!("Fetching Nix tags from GitHub...");
    let tags = fetch_nix_tags().context("Failed to fetch Nix tags")?;

    let latest_version =
        get_latest_stable_version(&tags).context("Failed to determine latest version")?;
    println!("Latest stable Nix version: {}", latest_version);

    let current_state = OutputVersions {
        nix_version: latest_version.clone(),
    };

    let mut nix_needs_update = true;

    // Try to read previous state
    if let Some(input_path) = &args.input {
        if input_path.exists() {
            println!("Reading previous state from {:?}", input_path);
            if let Ok(content) = fs::read_to_string(input_path) {
                if let Ok(previous_state) = serde_json::from_str::<OutputVersions>(&content) {
                    if previous_state.nix_version == current_state.nix_version {
                        nix_needs_update = false;
                        println!("Nix is up to date.");
                    } else {
                        println!(
                            "Nix update found: {} -> {}",
                            previous_state.nix_version, current_state.nix_version
                        );
                    }
                } else if content.trim() == "{}" {
                    println!("Previous state file is empty, assuming fresh start.");
                } else {
                    eprintln!("Failed to parse previous state file, assuming update needed.");
                }
            }
        }
    }

    // Generate output JSON for state recording
    let state_json = serde_json::to_string_pretty(&current_state)?;
    println!("Current State:\n{}", state_json);

    if let Some(path) = args.output {
        fs::write(path, state_json).context("Failed to write output file")?;
    }

    // Generate output JSON for update status
    if let Some(status_path) = args.update_status {
        let status = CheckResult {
            nix_needs_update,
            current: current_state,
        };
        let status_json = serde_json::to_string_pretty(&status)?;
        fs::write(&status_path, status_json).context("Failed to write status file")?;
        println!("Update status written to {:?}", status_path);
    }

    Ok(())
}
