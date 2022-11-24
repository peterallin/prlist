use anyhow::{Context, Result};
use clap::Parser;
use early::Early;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Reply<T> {
    value: Vec<T>,
}

#[derive(Debug, Deserialize)]
struct Person {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PullRequest {
    title: String,
    description: Option<String>,
    is_draft: bool,
}

#[derive(clap::Parser)]
struct Options {
    /// Path to the file containing the PAT for authenticating with Azure DevOps
    pat_file: std::path::PathBuf,
    /// Username on Azure DevOps
    username: String,
    /// Name of the Azure DevOps organization
    organization: String,
    /// Name of the team project in Azure DevOps
    project: String,
}

fn main() -> Result<()> {
    let options = Options::parse();

    let pat = std::fs::read_to_string(&options.pat_file)
        .with_context(|| format!("Failed to read PAT from {}", options.pat_file.display()))?;
    let client = reqwest::blocking::Client::new();
    let dev_api = Early::new("https", "dev.azure.com")
        .path(&options.organization)
        .path(&options.project)
        .path("_apis")
        .query("api_version", "7.0");

    let pull_requests = dev_api.path("git").path("pullrequests").build();

    let pull_requests: Reply<PullRequest> = client
        .get(pull_requests)
        .basic_auth(&options.username, Some(pat))
        .send()?
        .json()?;

    for pr in pull_requests.value.into_iter().filter(|pr| !pr.is_draft) {
        println!("{}", pr.title);
        if let Some(description) = pr.description {
            if description != pr.title {
                println!("\n{description}");
            }
            println!("------------");
        }
    }
    Ok(())
}
