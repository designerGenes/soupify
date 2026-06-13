use std::path::PathBuf;

use clap::Parser;

use crate::error::SoupifyError;
use crate::models::CliArgs;

#[derive(Debug, Parser)]
#[command(
    name = "soupify",
    version,
    about = "Combine files into a markdown soup"
)]
struct RawCliArgs {
    #[arg(short = 'o', long = "output")]
    output_dir: Option<PathBuf>,
    #[arg(short = 'd', long = "desoupify")]
    desoupify: bool,
    #[arg(short = 's', long = "show")]
    show_output_dir: bool,
    #[arg(short = 'r', long = "recursive")]
    recursive: bool,
    #[arg(short = 'x', long = "exclude")]
    exclude: Vec<String>,
    #[arg(value_name = "INPUT", required = true)]
    inputs: Vec<PathBuf>,
}

pub fn parse_cli_args() -> Result<CliArgs, SoupifyError> {
    parse_cli_args_from(std::env::args_os())
}

pub fn parse_cli_args_from<I, T>(args: I) -> Result<CliArgs, SoupifyError>
where
    I: IntoIterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let parsed = RawCliArgs::try_parse_from(args)
        .map_err(|error| SoupifyError::InvalidCliUsage(error.to_string()))?;

    if parsed.inputs.is_empty() {
        return Err(SoupifyError::InvalidCliUsage(
            "at least one input path is required".to_string(),
        ));
    }

    if parsed.desoupify && parsed.show_output_dir {
        return Err(SoupifyError::InvalidCliUsage(
            "-d/--desoupify cannot be combined with -s/--show".to_string(),
        ));
    }

    Ok(CliArgs {
        desoupify: parsed.desoupify,
        show_output_dir: parsed.show_output_dir,
        output_dir: parsed.output_dir,
        recursive: parsed.recursive,
        inputs: parsed.inputs,
        exclude: parsed.exclude,
    })
}

#[cfg(test)]
mod tests {
    use super::parse_cli_args_from;

    #[test]
    fn rejects_missing_inputs() {
        let result = parse_cli_args_from(["soupify"]);
        let error = result.expect_err("expected missing input failure");
        assert!(error.to_string().contains("required"));
    }

    #[test]
    fn rejects_desoupify_show_combination() {
        let result = parse_cli_args_from(["soupify", "-d", "-s", "file.txt"]);
        let error = result.expect_err("expected invalid flag combination");
        assert!(error.to_string().contains("cannot be combined"));
    }
}
