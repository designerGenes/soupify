use std::fs;
use std::path::PathBuf;

use crate::config::Config;
use crate::error::SoupifyError;
use crate::graph;
use crate::models::{CliArgs, SoupMetaBlock, SourceFile};
use crate::pathing::{
    build_output_filename, collect_source_files, filename_token, resolve_absolute,
    resolve_output_dir,
};
use crate::sharktopus;
use crate::soup_format::{analyze_contents, serialize_document};

pub fn run_soupify(args: &CliArgs, config: &Config) -> Result<PathBuf, SoupifyError> {
    let cwd = std::env::current_dir().map_err(|error| SoupifyError::FileReadFailure {
        path: PathBuf::from("."),
        source: error,
    })?;

    let output_dir = resolve_output_dir(
        args.output_dir
            .as_deref()
            .or(args.soupify_to.as_deref())
            .or(config.soupified_folder.as_deref()),
        &cwd,
    )?;
    let resolved_inputs = args
        .inputs
        .iter()
        .map(|input| resolve_absolute(input, &cwd))
        .collect::<Result<Vec<_>, _>>()?;

    for input in &resolved_inputs {
        if !input.exists() {
            return Err(SoupifyError::MissingInputPath(input.clone()));
        }
    }

    let max_depth = if args.recursive { Some(usize::MAX) } else { Some(0) };
    let files = collect_source_files(&resolved_inputs, max_depth, &args.exclude)?;
    if files.is_empty() {
        return Err(SoupifyError::InputExpandedToZeroFiles);
    }

    let source_files = files
        .iter()
        .map(build_source_file)
        .collect::<Result<Vec<_>, _>>()?;

    let meta_blocks = if graph::should_include_graph(args.include_graph, config) {
        build_graph_meta_blocks(&files, config)?
    } else {
        Vec::new()
    };

    let markdown = serialize_document(&meta_blocks, &source_files)?;

    if config.connect_with_downloads_watcher {
        match sharktopus::ensure_rules(config) {
            Ok(messages) => {
                for msg in &messages {
                    eprintln!("{msg}");
                }
            }
            Err(error) => {
                eprintln!("warning: failed to configure Sharktopus: {error}");
            }
        }
    }

    fs::create_dir_all(&output_dir).map_err(|error| SoupifyError::DirectoryCreationFailure {
        path: output_dir.clone(),
        source: error,
    })?;

    let output_file = output_dir.join(build_output_filename(&files)?);
    fs::write(&output_file, markdown).map_err(|error| SoupifyError::FileWriteFailure {
        path: output_file.clone(),
        source: error,
    })?;

    Ok(output_file)
}

fn build_graph_meta_blocks(
    files: &[PathBuf],
    config: &Config,
) -> Result<Vec<SoupMetaBlock>, SoupifyError> {
    let Some(repo_root) = graph::shared_git_root(files) else {
        eprintln!(
            "warning: --include-graph set but files do not share a single git repository; skipping graph"
        );
        return Ok(Vec::new());
    };

    let meta_block = graph::generate_repomap(&repo_root, files, config)?;
    Ok(vec![meta_block])
}

fn build_source_file(path: &PathBuf) -> Result<SourceFile, SoupifyError> {
    let bytes = fs::read(path).map_err(|error| SoupifyError::FileReadFailure {
        path: path.clone(),
        source: error,
    })?;
    let contents =
        String::from_utf8(bytes).map_err(|_| SoupifyError::Utf8DecodeFailure(path.clone()))?;
    let (logical_line_count, trailing_newline) = analyze_contents(&contents);

    Ok(SourceFile {
        original_absolute_path: path.clone(),
        file_name: path
            .file_name()
            .expect("collected file should have basename")
            .to_string_lossy()
            .to_string(),
        name_token: filename_token(path)?,
        contents,
        logical_line_count,
        trailing_newline,
    })
}
