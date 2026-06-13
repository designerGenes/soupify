use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliArgs {
    pub desoupify: bool,
    pub show_output_dir: bool,
    pub output_dir: Option<PathBuf>,
    pub recursive: bool,
    pub inputs: Vec<PathBuf>,
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFile {
    pub original_absolute_path: PathBuf,
    pub file_name: String,
    pub name_token: String,
    pub contents: String,
    pub logical_line_count: usize,
    pub trailing_newline: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoupPartialRange {
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoupBlock {
    pub original_absolute_path: PathBuf,
    pub partial_range: Option<SoupPartialRange>,
    pub logical_line_count: usize,
    pub trailing_newline: bool,
    pub content_lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoupDocument {
    pub blocks: Vec<SoupBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SoupMatchResult {
    One(PathBuf),
    None,
    Ambiguous(Vec<PathBuf>),
}
