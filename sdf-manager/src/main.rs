use std::{
    fs,
    io::{self, IsTerminal, Write},
};

use crate::cli::Cli;

mod cli;

use anyhow::Context;
use clap::Parser;
use reqwest::{Response, Url};
use sdf_data_structures::{model::SdfModel, supplement::SdfSupplement, traits::SdfDataStructure};

#[derive(PartialEq, Eq, Debug)]
struct BasicAuthCredentials {
    username: String,
    password: String,
}

fn prompt_username(prompt: impl ToString) -> anyhow::Result<String> {
    eprint!("{}", prompt.to_string());
    std::io::stdout().flush()?;

    let mut user_input = String::new();
    std::io::stdin().read_line(&mut user_input)?;

    Ok(user_input.trim_end_matches("\n").to_string())
}

fn get_basic_auth_credentials() -> anyhow::Result<BasicAuthCredentials> {
    eprintln!("This operation requires you to log into the SDF Repository.");

    let username = prompt_username("Username: ")?;
    let password = rpassword::prompt_password("Password: ")?;

    let credentials = BasicAuthCredentials { username, password };

    Ok(credentials)
}

fn print_bytes(bytes: &[u8]) -> anyhow::Result<()> {
    io::stdout().write_all(bytes)?;

    if std::io::stdout().is_terminal() {
        println!();
    }

    Ok(())
}

async fn handle_model_response(
    status_message: &'static str,
    response: Response,
) -> anyhow::Result<()> {
    let sdf_model = response.json::<SdfModel>().await?;

    eprintln!("{status_message}");

    let bytes = serde_json::to_vec(&sdf_model)?;

    print_bytes(&bytes)?;

    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.operation {
        cli::Operation::Register { input_file } => {
            let contents = fs::read_to_string(input_file)?;

            let sdf_model = serde_json::from_str::<SdfModel>(&contents)?;

            let BasicAuthCredentials { username, password } = get_basic_auth_credentials()?;

            let target_namespace = sdf_model
                .get_target_namespace()?
                .context("Target namespace is not defined!")?;

            let mut sdf_repository_url = Url::parse(&target_namespace)?;

            sdf_repository_url.set_path("/api/models");

            let response = reqwest::Client::new()
                .post(sdf_repository_url)
                .basic_auth(username, Some(password))
                .header("content-type", "application/sdf+json")
                .body(contents)
                .send()
                .await?;

            response.error_for_status_ref()?;

            if let Some(location) = response.headers().get("Location") {
                eprintln!(
                    "New model has been registered under {}.",
                    location.to_str()?
                );
            }

            handle_model_response(
                "Received representation of registered model from SDF Repository, writing to stdout.",
                response,
            ).await?
        }
        cli::Operation::List {
            target_namespace,
            lineage,
            version,
            min_version,
            max_version,
            exclusive_max_version,
            exclusive_min_version,
        } => {
            let mut repository_url = Url::parse(&target_namespace)?;
            repository_url.set_path("/api/models");

            repository_url
                .query_pairs_mut()
                .clear()
                .append_pair("namespace", &target_namespace);

            for (key, value) in [
                ("lineage", lineage),
                ("version", version),
                ("minVersion", min_version),
                ("maxVersion", max_version),
                ("exclusiveMaxVersion", exclusive_min_version),
                ("exclusiveMinVersion", exclusive_max_version),
            ] {
                if let Some(value) = value {
                    repository_url.query_pairs_mut().append_pair(key, &value);
                }
            }

            let response = reqwest::get(repository_url).await?;

            let bytes = response.bytes().await?;

            print_bytes(&bytes)?;
        }
        cli::Operation::Update { input_file } => {
            let contents = fs::read_to_string(input_file)?;

            let sdf_supplement = serde_json::from_str::<SdfSupplement>(&contents)?;

            let BasicAuthCredentials { username, password } = get_basic_auth_credentials()?;

            let target_namespace = sdf_supplement
                .get_target_namespace()?
                .context("Target namespace is not defined!")?;

            let mut sdf_repository_url = Url::parse(&target_namespace)?;

            sdf_repository_url.set_path("/api/supplements");

            let response = reqwest::Client::new()
                .post(sdf_repository_url)
                .basic_auth(username, Some(password))
                .header("content-type", "application/sdf-supplement+json")
                .body(contents)
                .send()
                .await?;

            response.error_for_status_ref()?;

            if let Some(location) = response.headers().get("Location") {
                eprintln!("Updated model is available under {}.", location.to_str()?);
            }

            handle_model_response(
                "Received representation of updated model from SDF Repository, writing to stdout.",
                response,
            )
            .await?
        }
        cli::Operation::Delete {
            target_namespace,
            lineage,
            min_version,
        } => {
            let BasicAuthCredentials { username, password } = get_basic_auth_credentials()?;

            let mut url = Url::parse(target_namespace.as_str())?;

            url.query_pairs_mut().clear();

            for (key, value) in [("lineage", lineage), ("minVersion", min_version)] {
                if let Some(value) = value {
                    url.query_pairs_mut().append_pair(key, &value);
                }
            }

            let response = reqwest::Client::new()
                .delete(url)
                .basic_auth(username, Some(password))
                .send()
                .await?;

            response.error_for_status_ref()?;

            eprintln!("Resource deletion succeeded.");

            let bytes = response.bytes().await?;

            print_bytes(&bytes)?;
        }
    }

    Ok(())
}
