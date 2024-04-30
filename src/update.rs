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
use std::{env::current_exe, path::PathBuf, process::exit};

/// The GitHub repository to use for releases
pub const GITHUB_REPOSITORY: &str = "PocketRelay/Client";
// Windows non native asset name
#[cfg(all(target_family = "windows", not(feature = "native")))]
pub const ASSET_NAME: &str = "pocket-relay-client.exe";
// Windows native asset name
#[cfg(all(target_family = "windows", feature = "native"))]
pub const ASSET_NAME: &str = "pocket-relay-client-native.exe";
// Linux asset name
#[cfg(target_family = "unix")]
pub const ASSET_NAME: &str = "pocket-relay-client-linux";

/// Paths used by the updater
pub struct UpdatePaths {
    /// Path to the file executable file
    pub exe: PathBuf,
    /// Temporary path for storing the file while download
    pub tmp_download: PathBuf,
    /// Temporary path for moving the old plugin before swapping
    pub tmp_old: PathBuf,
}

impl Default for UpdatePaths {
    fn default() -> Self {
        // Locate the executable path
        let path = current_exe().expect("Unable to locate executable path");
        // Find the parent directory of the executable
        let parent = path.parent().expect("Missing exe parent directory");

        Self {
            exe: path.clone(),
            tmp_download: parent.join("pocket-relay-client.exe.tmp-download"),
            tmp_old: parent.join("pocket-relay-client.exe.tmp-old"),
        }
    }
}

impl UpdatePaths {
    // Removes the temporary paths if they exist
    pub async fn remove_tmp_paths(&self) -> std::io::Result<()> {
        if self.tmp_old.exists() {
            tokio::fs::remove_file(&self.tmp_old).await?;
        }

        if self.tmp_download.exists() {
            tokio::fs::remove_file(&self.tmp_download).await?;
        }

        Ok(())
    }

    /// Moves the `plugin` file to `tmp_old` and moves the downloaded
    /// file from `tmp_download` to `plugin`
    pub async fn swap_plugin_files(&self) -> std::io::Result<()> {
        debug!("Swapping plugin files with update");

        // Move the exe to the `tmp_old` path
        tokio::fs::rename(&self.exe, &self.tmp_old).await?;

        // Move the downloaded plugin to the `exe` path
        tokio::fs::rename(&self.tmp_download, &self.exe).await?;

        Ok(())
    }
}

/// Handles the updating process
pub async fn update(http_client: reqwest::Client) {
    let paths = UpdatePaths::default();

    // Remove temporary files if they exist
    if let Err(err) = paths.remove_tmp_paths().await {
        error!("Failed to remove temporary files: {}", err);
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

    // Don't update if we are already on the latest or an unreleased version
    if current_version >= latest_version {
        if current_version > latest_version {
            debug!("Future release is installed ({})", current_version);
        } else {
            debug!("Latest version is installed ({})", current_version);
        }

        return;
    }

    debug!("New version is available ({})", latest_version);

    let Some(asset) = latest_release
        .assets
        .iter()
        .find(|asset| asset.name == ASSET_NAME)
    else {
        error!("Server release is missing the desired binary, cannot update");
        return;
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

    let bytes = match download_latest_release(&http_client, asset).await {
        Ok(bytes) => bytes,
        Err(err) => {
            show_error("Failed to download", &err.to_string());

            // Delete partially downloaded file if present
            if let Err(err) = paths.remove_tmp_paths().await {
                error!("Failed to remove temporary files: {}", err);
            }

            return;
        }
    };

    // Save the downloaded file to the tmp path
    if let Err(err) = tokio::fs::write(&paths.tmp_download, bytes).await {
        show_error("Failed to save downloaded update", &err.to_string());
        return;
    }

    // Swap the plugin files with the new version
    if let Err(err) = paths.swap_plugin_files().await {
        error!("Failed to swap plugin files: {}", err);
    }

    show_info(
        "Update successful",
        "The client has been updated, restart the client now to use the new version",
    );

    exit(0);
}
