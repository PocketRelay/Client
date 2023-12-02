//! Updater module for providing auto-updating functionality

use crate::{
    core::{
        reqwest,
        update::{download_latest_release, get_latest_release},
        Version,
    },
    ui::{show_confirm, show_error, show_info},
    APP_VERSION,
};
use log::{debug, error};
use std::{env::current_exe, process::exit};

/// The GitHub repository to use for releases
pub const GITHUB_REPOSITORY: &str = "PocketRelay/Client";

/// Handles the updating process
pub async fn update(http_client: reqwest::Client) {
    let path = current_exe().expect("Unable to locate executable path");

    let parent = path.parent().expect("Missing exe parent directory");

    let tmp_file = parent.join("pocket-relay-client.exe.tmp-download");
    let tmp_old = parent.join("pocket-relay-client.exe.tmp-old");

    // Remove the old file if it exists
    if tmp_old.exists() {
        tokio::fs::remove_file(&tmp_old)
            .await
            .expect("Failed to remove old executable");
    }

    // Remove temp download file if it exists
    if tmp_file.exists() {
        tokio::fs::remove_file(&tmp_file)
            .await
            .expect("Failed to remove temp executable");
    }

    debug!("Checking for updates");
    let latest_release = match get_latest_release(&http_client, GITHUB_REPOSITORY).await {
        Ok(value) => value,
        Err(err) => {
            error!("Failed to fetch latest release: {}", err);
            return;
        }
    };

    let latest_version = latest_release
        .tag_name
        .trim_start_matches('v')
        .parse::<Version>();

    let latest_version = match latest_version {
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

    // Windows non native asset name
    #[cfg(all(target_family = "windows", not(feature = "native")))]
    let asset_name = "pocket-relay-client.exe";

    // Windows native asset name
    #[cfg(all(target_family = "windows", feature = "native"))]
    let asset_name = "pocket-relay-client-native.exe";

    // Linux asset name
    #[cfg(target_family = "unix")]
    let asset_name = "pocket-relay-client-linux";

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

    if !show_confirm("New version is available", &msg) {
        return;
    }

    debug!("Downloading release");

    match download_latest_release(&http_client, asset).await {
        Ok(bytes) => {
            // Save the downloaded file to the tmp path
            if let Err(err) = tokio::fs::write(&tmp_file, bytes).await {
                show_error("Failed to save downloaded update", &err.to_string());
                return;
            }
        }
        Err(err) => {
            show_error("Failed to download", &err.to_string());

            // Delete partially downloaded file if present
            if tmp_file.exists() {
                let _ = tokio::fs::remove_file(tmp_file).await;
            }

            return;
        }
    }

    debug!("Swapping executable files");

    tokio::fs::rename(&path, &tmp_old)
        .await
        .expect("Failed to rename executable to temp path");
    tokio::fs::rename(&tmp_file, path)
        .await
        .expect("Failed to rename executable");

    show_info(
        "Update successfull",
        "The client has been updated, restart the client now to use the new version",
    );

    exit(0);
}
