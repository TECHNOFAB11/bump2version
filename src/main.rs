use self::cli::Cli;
use crate::config::Config;
use crate::utils::attempt_version_bump;
use clap::Parser;
use std::fs;
use std::process::{exit, Command};
use tracing::{error, info, level_filters::LevelFilter, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod cli;
mod config;
mod utils;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    let args = Cli::parse();
    let config_file = args.config_file.clone();
    let contents = match fs::read_to_string(&config_file) {
        Ok(c) => c,
        Err(err) => {
            error!(?err, config_file, "Could not read config");
            exit(1);
        }
    };
    let config: Config = match toml::from_str(&contents) {
        Ok(d) => d,
        Err(err) => {
            error!(err = err.to_string(), config_file, "Unable to load config");
            exit(1);
        }
    };

    let current_version = config.current_version.clone();

    let attempted_new_version = args
        .new_version
        .clone()
        .or(attempt_version_bump(args.clone(), config.clone()));

    if attempted_new_version.is_some() {
        let new_version = attempted_new_version.clone().unwrap();
        info!(current_version, new_version, "Bumping version");

        let dry_run = args.dry_run;
        let commit = args.commit.unwrap_or(config.commit);
        let tag = args.tag.unwrap_or(config.tag);
        let message = args
            .message
            .or(config.message)
            .unwrap_or("Bump version: {current_version} → {new_version}".to_string());

        let config_files = config.file.unwrap_or_default();
        let files: Vec<&String> = config_files.keys().collect();

        // Check if Git working directory is clean
        if fs::metadata(".git").is_ok() {
            let git_status = Command::new("git")
                .arg("status")
                .arg("--porcelain")
                .output()?;

            let git_output = String::from_utf8_lossy(&git_status.stdout);
            let git_lines: Vec<&str> = git_output
                .lines()
                .filter(|line| !line.trim().starts_with("??"))
                .collect();

            if !git_lines.is_empty() {
                warn!("Git working directory not clean:\n{}", git_lines.join("\n"));
                if args.fail_on_dirty {
                    exit(1);
                }
            }
        }

        // Update version in specified files
        info!(amount = &files.len(), "Updating version in files");
        for path in files.clone() {
            let content = fs::read_to_string(path)?;
            let format = &config_files.get(path).unwrap().format.clone();

            let old_line = format.replace("{version}", &current_version);
            if !content.contains(&old_line) {
                warn!(
                    current_version,
                    path, "Did not find current version in file"
                );
            }

            let new_line = format.replace("{version}", &new_version);
            let updated_content = content.replace(&old_line, &new_line);

            if !dry_run {
                fs::write(path, updated_content)?;
            }
        }

        let mut commit_files = files.clone();

        // Update config file if applicable & writable
        let metadata = fs::metadata(config_file.clone());
        if metadata.is_ok() && !metadata.unwrap().permissions().readonly() {
            info!("Updating version in config file");
            let mut config_content = fs::read_to_string(config_file.clone())?;

            config_content = config_content.replace(
                &format!("current_version = \"{}\"", current_version),
                &format!("current_version = \"{}\"", new_version),
            );

            if !dry_run {
                fs::write(config_file.clone(), config_content)?;
                commit_files.push(&config_file);
            }
        }
        if commit {
            for path in &commit_files {
                let git_add_output = Command::new("git").arg("add").arg(path).output();

                match git_add_output {
                    Ok(output) => {
                        if !output.status.success() {
                            let stderr = String::from_utf8_lossy(&output.stderr);
                            error!(?stderr, "Error during git add");
                        }
                    }
                    Err(err) => {
                        error!(?err, "Failed to execute git add");
                    }
                }
            }
            let git_diff_output = Command::new("git").arg("diff").output();

            match git_diff_output {
                Ok(output) => {
                    if output.stdout.is_empty() {
                        let commit_output = Command::new("git")
                            .arg("commit")
                            .arg("-m")
                            .arg(
                                message
                                    .replace("{current_version}", &current_version)
                                    .replace("{new_version}", &new_version),
                            )
                            .output();

                        match commit_output {
                            Ok(commit_output) => {
                                if commit_output.status.success() {
                                    info!("Git commit successful");
                                } else {
                                    let stderr = String::from_utf8_lossy(&commit_output.stderr);
                                    error!(?stderr, "Error during git commit",);
                                }
                            }
                            Err(err) => {
                                error!(?err, "Failed to execute git commit");
                            }
                        }
                    } else {
                        warn!("No changes to commit. Working tree clean.");
                    }
                }
                Err(err) => {
                    error!(?err, "Failed to execute git diff");
                }
            }

            if tag {
                Command::new("git")
                    .arg("tag")
                    .arg(format!("v{}", new_version))
                    .output()?;
            }
        }
    } else {
        error!("No new version passed and generating new version failed");
    }

    Ok(())
}
