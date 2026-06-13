use std::fs;
use std::path::Path;
use std::process::Command;

use crate::error::SoupifyError;

pub trait OutputDirOpener {
    fn open(&self, path: &Path) -> Result<(), SoupifyError>;
}

#[derive(Debug, Default)]
pub struct SystemOutputDirOpener;

pub fn open_output_dir_with(
    opener: &impl OutputDirOpener,
    path: &Path,
) -> Result<(), SoupifyError> {
    opener.open(path)
}

impl OutputDirOpener for SystemOutputDirOpener {
    fn open(&self, path: &Path) -> Result<(), SoupifyError> {
        if let Some(mock_file) = std::env::var_os("SOUPIFY_OPEN_MOCK_FILE") {
            fs::write(&mock_file, format!("{}\n", path.display())).map_err(|error| {
                SoupifyError::OpenDirectoryFailure {
                    directory: path.to_path_buf(),
                    message: error.to_string(),
                }
            })?;
        }

        if std::env::var_os("SOUPIFY_OPEN_FORCE_FAIL").is_some() {
            return Err(SoupifyError::OpenDirectoryFailure {
                directory: path.to_path_buf(),
                message: "forced open failure".to_string(),
            });
        }

        if std::env::var_os("SOUPIFY_OPEN_MOCK_FILE").is_some() {
            return Ok(());
        }

        let command = if cfg!(target_os = "macos") {
            "open"
        } else if cfg!(target_os = "windows") {
            "explorer"
        } else {
            "xdg-open"
        };

        let status = Command::new(command).arg(path).status().map_err(|error| {
            SoupifyError::OpenDirectoryFailure {
                directory: path.to_path_buf(),
                message: error.to_string(),
            }
        })?;

        if status.success() {
            Ok(())
        } else {
            Err(SoupifyError::OpenDirectoryFailure {
                directory: path.to_path_buf(),
                message: format!("{command} exited with status {status}"),
            })
        }
    }
}
