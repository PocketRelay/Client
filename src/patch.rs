//! Patching module providing game patching functionality

use crate::{
    constants::{BINKW23_DLL_BYTES, BINKW32_DLL_BYTES},
    ui::try_pick_game_path,
};
use std::{
    fs::{copy, remove_file, write},
    io,
};
use thiserror::Error;

/// Errors that can occur while patching the game
#[derive(Debug, Error)]
pub enum PatchError {
    /// The file picker failed to pick a file
    #[error("Failed to get picked file. Make sure this program is running as administrator")]
    PickFileFailed,
    /// The picked path was missing the game exe
    #[error("The path given doesn't contains the MassEffect.exe executable")]
    MissingGame,
    /// Failed to delete the bink232
    #[error("Failed to delete binkw32.dll you will have to manually unpatch your game: {0}")]
    FailedDelete(io::Error),
    /// Fialed to replace the files
    #[error("Failed to replace binkw32.dll with origin binkw23.ddl: {0}")]
    FailedReplaceOriginal(io::Error),
    /// Failed to write the patch files
    #[error("Failed to write patch file dlls (binkw32.dll and binkw32.dll): {0}")]
    FailedWritingPatchFiles(io::Error),
}

/// Attempts to remove the patch from a game, prompts the user for the
/// game executable path then attempts to remove the patches
pub fn try_remove_patch() -> Result<bool, PatchError> {
    let path = match try_pick_game_path().map_err(|_| PatchError::PickFileFailed)? {
        Some(value) => value,
        None => return Ok(false),
    };
    if !path.exists() {
        return Err(PatchError::MissingGame);
    }

    let parent = path.parent().ok_or(PatchError::MissingGame)?;

    let binkw23 = parent.join("binkw23.dll");
    let binkw32 = parent.join("binkw32.dll");

    if binkw32.exists() {
        remove_file(&binkw32).map_err(PatchError::FailedDelete)?;
    }

    if binkw23.exists() {
        copy(&binkw23, &binkw32).map_err(PatchError::FailedReplaceOriginal)?;
        let _ = remove_file(&binkw23);
    } else {
        write(&binkw32, BINKW23_DLL_BYTES).map_err(PatchError::FailedReplaceOriginal)?;
    }

    Ok(true)
}

/// Attempts to patch a game, prompts the user for the game executable
/// path then attempts to write the patches
pub fn try_patch_game() -> Result<bool, PatchError> {
    let path = match try_pick_game_path().map_err(|_| PatchError::PickFileFailed)? {
        Some(value) => value,
        None => return Ok(false),
    };
    if !path.exists() {
        return Err(PatchError::MissingGame);
    }
    let parent = path.parent().ok_or(PatchError::MissingGame)?;

    let binkw23 = parent.join("binkw23.dll");
    let binkw32 = parent.join("binkw32.dll");

    write(binkw23, BINKW23_DLL_BYTES).map_err(PatchError::FailedWritingPatchFiles)?;
    write(binkw32, BINKW32_DLL_BYTES).map_err(PatchError::FailedWritingPatchFiles)?;
    Ok(true)
}
