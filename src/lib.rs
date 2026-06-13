pub mod cli;
pub mod desoupify;
pub mod error;
pub mod models;
pub mod open;
pub mod pathing;
pub mod soup_format;
pub mod soupify;

use cli::parse_cli_args;
use error::SoupifyError;
use open::{OutputDirOpener, SystemOutputDirOpener};

pub fn run() -> Result<(), SoupifyError> {
    let args = parse_cli_args()?;
    run_with_opener(&args, &SystemOutputDirOpener)
}

pub fn run_with_opener(
    args: &models::CliArgs,
    opener: &impl OutputDirOpener,
) -> Result<(), SoupifyError> {
    if args.desoupify {
        desoupify::run_desoupify(args)?;
        return Ok(());
    }

    let soup_file = soupify::run_soupify(args)?;
    if args.show_output_dir {
        let output_dir = soup_file
            .parent()
            .map(|path| path.to_path_buf())
            .ok_or_else(|| SoupifyError::OpenDirectoryAfterWriteFailed {
                soup_file: soup_file.clone(),
                directory: soup_file.clone(),
                message: "generated soup file has no parent directory".to_string(),
            })?;

        if let Err(error) = open::open_output_dir_with(opener, &output_dir) {
            return Err(SoupifyError::OpenDirectoryAfterWriteFailed {
                soup_file,
                directory: output_dir,
                message: error.to_string(),
            });
        }
    }

    Ok(())
}
