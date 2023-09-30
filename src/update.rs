//! Updater module for providing auto-updating functionality

use crate::{
    constants::{APP_VERSION, IS_NATIVE_VERSION, PR_USER_AGENT},
    ui::{show_confirm, show_error, show_info},
};
use log::{debug, error};
use reqwest::header::{ACCEPT, USER_AGENT};
use semver::Version;
use serde::Deserialize;
use std::{env::current_exe, path::Path, process::exit};

/// Structure for https://api.github.com/repos/PocketRelay/Client/releases/latest
/// (Only the required parts)
#[derive(Debug, Deserialize)]
pub struct GitHubRelease {
    /// The URL for viewing the release in the browser
    pub html_url: String,
    /// The release tag / version
    pub tag_name: String,
    /// The name of the release (Usually the same as tag_name)
    pub name: String,
    /// The datetime the release was published
    pub published_at: String,

    pub assets: Vec<GitHubReleaseAsset>,
}

#[derive(Debug, Deserialize)]
pub struct GitHubReleaseAsset {
    /// The name of the file
    pub name: String,
    /// URL for downloading the file
    pub browser_download_url: String,
}

/// Attempts to obtain the latest release from github
pub async fn get_latest_release() -> Result<GitHubRelease, reqwest::Error> {
    let client = reqwest::Client::new();

    client
        .get("https://api.github.com/repos/PocketRelay/Client/releases/latest")
        .header(ACCEPT, "application/json")
        .header(USER_AGENT, PR_USER_AGENT)
        .send()
        .await?
        .json()
        .await
}

/// Attempts to download the latest release executable and
/// write it to the provided path
pub async fn download_latest_release(
    asset: &GitHubReleaseAsset,
    path: &Path,
) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let bytes = client
        .get(&asset.browser_download_url)
        .header(USER_AGENT, PR_USER_AGENT)
        .send()
        .await?
        .bytes()
        .await?;

    tokio::fs::write(path, bytes)
        .await
        .expect("Failed to write file");
    Ok(())
}

/// Handles the updating process
pub async fn update() {
    debug!("Checking for updates");
    let latest_release = match get_latest_release().await {
        Ok(value) => value,
        Err(err) => {
            error!("Failed to fetch latest release: {}", err);
            return;
        }
    };

    let latest_tag = latest_release
        .tag_name
        .strip_prefix('v')
        .unwrap_or(&latest_release.tag_name);

    let latest_version = match Version::parse(latest_tag) {
        Ok(value) => value,
        Err(err) => {
            error!("Failed to parse version of latest release: {}", err);
            return;
        }
    };

    let current_version = Version::parse(APP_VERSION).expect("Failed to parse app version");

    if latest_version <= current_version {
        if current_version > latest_version {
            debug!("Future release is installed ({})", current_version);
        } else {
            debug!("Latest version is installed ({})", current_version);
        }

        return;
    }

    debug!("New version is available ({})", latest_version);

    let asset_name = if IS_NATIVE_VERSION {
        "pocket-relay-client-native.exe"
    } else {
        "pocket-relay-client.exe"
    };

    let asset = match latest_release
        .assets
        .iter()
        .find(|asset| asset.name == asset_name)
    {
        Some(value) => value,
        None => {
            error!("Server release is missing the desired binary, cannot update");
            return;
        }
    };

    let msg = format!(
        "There is a new version of the client available, would you like to update automatically?\n\n\
        Your version: v{}\n\
        Latest Version: v{}\n",
        current_version, latest_version,
    );

    let confirm = show_confirm("New version is available", &msg);
    if !confirm {
        return;
    }

    let path = current_exe().expect("Unable to locate executable path");

    let parent = path.parent().expect("Missing exe parent directory");

    let tmp_file = parent.join("pocket-relay-client.exe.tmp-download");
    let tmp_old = parent.join("pocket-relay-client.exe.tmp-old");

    debug!("Downloading release");

    if let Err(err) = download_latest_release(asset, &tmp_file).await {
        show_error("Failed to download", &err.to_string());
        if tmp_file.exists() {
            let _ = tokio::fs::remove_file(tmp_file).await;
        }

        return;
    }

    debug!("Swapping executable files");

    tokio::fs::rename(&path, &tmp_old)
        .await
        .expect("Failed to rename executable to temp path");
    tokio::fs::rename(&tmp_file, path)
        .await
        .expect("Failed to rename executable");

    tokio::fs::remove_file(tmp_old)
        .await
        .expect("Failed to remove old executable");

    show_info(
        "Update successfull",
        "The client has been updated, restart the client now to use the new version",
    );

    exit(0);
}
