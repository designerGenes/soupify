pub mod cli;
pub mod config;
pub mod desoupify;
pub mod error;
pub mod graph;
pub mod models;
pub mod open;
pub mod pathing;
pub mod repomap;
pub mod sharktopus;
pub mod soup_format;
pub mod soupify;

use cli::parse_cli_args;
use config::load_config;
use error::SoupifyError;
use open::{OutputDirOpener, SystemOutputDirOpener};

pub fn run() -> Result<(), SoupifyError> {
    if let Err(error) = config::ensure_config_dir() {
        eprintln!("warning: failed to create config directory: {error}");
    }
    let args = parse_cli_args()?;
    let config = load_config();
    run_with_opener(&args, &config, &SystemOutputDirOpener)
}

pub fn run_with_opener(
    args: &models::CliArgs,
    config: &config::Config,
    opener: &impl OutputDirOpener,
) -> Result<(), SoupifyError> {
    if args.desoupify {
        desoupify::run_desoupify(args, config)?;
        return Ok(());
    }

    let soup_file = soupify::run_soupify(args, config)?;
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
