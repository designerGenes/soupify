use std::path::PathBuf;
use std::sync::OnceLock;

use regex::Regex;

use crate::error::SoupifyError;
use crate::models::{SoupBlock, SoupDocument, SoupPartialRange, SourceFile};

pub fn serialize_document(files: &[SourceFile]) -> Result<String, SoupifyError> {
    let mut lines = Vec::new();

    for file in files {
        lines.push(serialize_header(file)?);
        for line in content_lines(&file.contents) {
            lines.push(line);
        }
    }

    Ok(lines.join("\n"))
}

pub fn parse_document(markdown: &str) -> Result<SoupDocument, SoupifyError> {
    if markdown.is_empty() {
        return Ok(SoupDocument { blocks: Vec::new() });
    }

    let lines: Vec<&str> = markdown.split('\n').collect();

    let mut blocks = Vec::new();
    let mut index = 0usize;

    while index < lines.len() {
        let header = lines[index];
        if header.is_empty() && index == lines.len() - 1 {
            break;
        }
        let (path, partial_range, logical_line_count, trailing_newline) =
            parse_header(header, index + 1)?;
        index += 1;

        if lines.len().saturating_sub(index) < logical_line_count {
            return Err(SoupifyError::SoupParseFailure(format!(
                "declared line count {} exceeds available lines for {}",
                logical_line_count,
                path.display()
            )));
        }

        let mut content_lines = Vec::with_capacity(logical_line_count);
        for _ in 0..logical_line_count {
            content_lines.push(lines[index].to_string());
            index += 1;
        }

        blocks.push(SoupBlock {
            original_absolute_path: path,
            partial_range,
            logical_line_count,
            trailing_newline,
            content_lines,
        });
    }

    Ok(SoupDocument { blocks })
}

fn serialize_header(file: &SourceFile) -> Result<String, SoupifyError> {
    let path = serde_json::to_string(&file.original_absolute_path.to_string_lossy().to_string())
        .map_err(|error| SoupifyError::SoupParseFailure(error.to_string()))?;

    Ok(format!(
        "#SOUP {path} #SOUPED_LINES {} #SOUP_TRAILING_NEWLINE {}",
        file.logical_line_count,
        usize::from(file.trailing_newline)
    ))
}

pub(crate) fn content_lines(contents: &str) -> Vec<String> {
    if contents.is_empty() {
        return Vec::new();
    }

    let trailing_newline = contents.ends_with('\n');
    let body = if trailing_newline {
        &contents[..contents.len() - 1]
    } else {
        contents
    };

    if body.is_empty() {
        return vec![String::new()];
    }

    body.split('\n').map(ToString::to_string).collect()
}

pub fn analyze_contents(contents: &str) -> (usize, bool) {
    let lines = content_lines(contents);
    (
        lines.len(),
        contents.ends_with('\n') && !contents.is_empty(),
    )
}

fn parse_header(
    line: &str,
    line_number: usize,
) -> Result<(PathBuf, Option<SoupPartialRange>, usize, bool), SoupifyError> {
    let path_captures = header_path_regex().captures(line).ok_or_else(|| {
        SoupifyError::SoupParseFailure(format!(
            "malformed soup header on line {line_number}: {line}"
        ))
    })?;

    let has_line_count_metadata = line.contains(" #SOUPED_LINES ");
    let has_trailing_newline_metadata = line.contains(" #SOUP_TRAILING_NEWLINE ");
    if (has_line_count_metadata || has_trailing_newline_metadata)
        && !(has_line_count_metadata && has_trailing_newline_metadata)
    {
        return Err(SoupifyError::SoupParseFailure(format!(
            "missing soup metadata on line {line_number}: {line}"
        )));
    }

    let captures = header_regex().captures(line).ok_or_else(|| {
        SoupifyError::SoupParseFailure(format!(
            "malformed soup header on line {line_number}: {line}"
        ))
    })?;

    let path_json = path_captures
        .get(1)
        .map(|capture| capture.as_str())
        .ok_or_else(|| {
            SoupifyError::SoupParseFailure(format!(
                "soup header missing path metadata on line {line_number}"
            ))
        })?;
    let path: String = serde_json::from_str(path_json).map_err(|error| {
        SoupifyError::SoupParseFailure(format!(
            "invalid escaped soup path on line {line_number}: {error}"
        ))
    })?;

    let partial_range = match (captures.get(2), captures.get(3)) {
        (Some(start), Some(end)) => {
            let start_line = start.as_str().parse::<usize>().map_err(|error| {
                SoupifyError::SoupParseFailure(format!(
                    "invalid partial soup start line on line {line_number}: {error}"
                ))
            })?;
            let end_line = end.as_str().parse::<usize>().map_err(|error| {
                SoupifyError::SoupParseFailure(format!(
                    "invalid partial soup end line on line {line_number}: {error}"
                ))
            })?;

            if start_line == 0 || end_line == 0 || start_line > end_line {
                return Err(SoupifyError::SoupParseFailure(format!(
                    "invalid partial soup range on line {line_number}: {line}"
                )));
            }

            Some(SoupPartialRange {
                start_line,
                end_line,
            })
        }
        (None, None) => None,
        _ => {
            return Err(SoupifyError::SoupParseFailure(format!(
                "malformed soup header on line {line_number}: {line}"
            )));
        }
    };

    let logical_line_count = captures
        .get(4)
        .ok_or_else(|| {
            SoupifyError::SoupParseFailure(format!(
                "soup header missing line count on line {line_number}"
            ))
        })?
        .as_str()
        .parse::<usize>()
        .map_err(|error| {
            SoupifyError::SoupParseFailure(format!(
                "invalid soup line count on line {line_number}: {error}"
            ))
        })?;

    let trailing_newline = match captures.get(5).map(|capture| capture.as_str()) {
        Some("0") | Some("false") => false,
        Some("1") | Some("true") => true,
        Some(other) => {
            return Err(SoupifyError::SoupParseFailure(format!(
                "invalid trailing newline marker on line {line_number}: {other}"
            )));
        }
        None => {
            return Err(SoupifyError::SoupParseFailure(format!(
                "soup header missing trailing newline metadata on line {line_number}"
            )));
        }
    };

    Ok((
        PathBuf::from(path),
        partial_range,
        logical_line_count,
        trailing_newline,
    ))
}

fn header_path_regex() -> &'static Regex {
    static HEADER_PATH_REGEX: OnceLock<Regex> = OnceLock::new();
    HEADER_PATH_REGEX.get_or_init(|| {
        Regex::new(r#"^#SOUP ("(?:\\.|[^"\\])*")(?: .*)?$"#)
            .expect("header path regex should compile")
    })
}

fn header_regex() -> &'static Regex {
    static HEADER_REGEX: OnceLock<Regex> = OnceLock::new();
    HEADER_REGEX.get_or_init(|| {
        Regex::new(
            r#"^#SOUP ("(?:\\.|[^"\\])*")(?: #SOUP_PARTIAL_LINES ([0-9]+)-([0-9]+))? #SOUPED_LINES ([0-9]+) #SOUP_TRAILING_NEWLINE (0|1|true|false)$"#,
        )
        .expect("header regex should compile")
    })
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::models::{SoupPartialRange, SourceFile};

    use super::{analyze_contents, parse_document, serialize_document};

    fn source_file(path: &str, contents: &str) -> SourceFile {
        let (logical_line_count, trailing_newline) = analyze_contents(contents);
        SourceFile {
            original_absolute_path: PathBuf::from(path),
            file_name: PathBuf::from(path)
                .file_name()
                .expect("basename should exist")
                .to_string_lossy()
                .to_string(),
            name_token: "token".to_string(),
            contents: contents.to_string(),
            logical_line_count,
            trailing_newline,
        }
    }

    #[test]
    fn serializes_a_single_block_correctly() {
        let document = serialize_document(&[source_file("/tmp/file.txt", "hello")])
            .expect("document should serialize");
        assert_eq!(
            document,
            "#SOUP \"/tmp/file.txt\" #SOUPED_LINES 1 #SOUP_TRAILING_NEWLINE 0\nhello"
        );
    }

    #[test]
    fn serializes_multiple_blocks_correctly() {
        let document = serialize_document(&[
            source_file("/tmp/one.txt", "one\n"),
            source_file("/tmp/two.txt", "two"),
        ])
        .expect("document should serialize");

        assert_eq!(
            document,
            "#SOUP \"/tmp/one.txt\" #SOUPED_LINES 1 #SOUP_TRAILING_NEWLINE 1\none\n#SOUP \"/tmp/two.txt\" #SOUPED_LINES 1 #SOUP_TRAILING_NEWLINE 0\ntwo"
        );
    }

    #[test]
    fn parses_valid_headers() {
        let document = parse_document(
            "#SOUP \"/tmp/file.txt\" #SOUPED_LINES 2 #SOUP_TRAILING_NEWLINE 0\na\nb",
        )
        .expect("document should parse");

        assert_eq!(document.blocks.len(), 1);
        assert_eq!(
            document.blocks[0].original_absolute_path,
            PathBuf::from("/tmp/file.txt")
        );
        assert_eq!(document.blocks[0].logical_line_count, 2);
        assert!(!document.blocks[0].trailing_newline);
        assert_eq!(document.blocks[0].partial_range, None);
    }

    #[test]
    fn parses_partial_headers() {
        let document = parse_document(
            "#SOUP \"/tmp/file.txt\" #SOUP_PARTIAL_LINES 2-3 #SOUPED_LINES 2 #SOUP_TRAILING_NEWLINE 1\na\nb",
        )
        .expect("document should parse");

        assert_eq!(document.blocks.len(), 1);
        assert_eq!(
            document.blocks[0].partial_range,
            Some(SoupPartialRange {
                start_line: 2,
                end_line: 3,
            })
        );
    }

    #[test]
    fn parses_boolean_trailing_newline_markers() {
        let with_newline = parse_document(
            "#SOUP \"/tmp/true.txt\" #SOUPED_LINES 1 #SOUP_TRAILING_NEWLINE true\na",
        )
        .expect("document should parse");
        let without_newline = parse_document(
            "#SOUP \"/tmp/false.txt\" #SOUPED_LINES 1 #SOUP_TRAILING_NEWLINE false\na",
        )
        .expect("document should parse");

        assert!(with_newline.blocks[0].trailing_newline);
        assert!(!without_newline.blocks[0].trailing_newline);
    }

    #[test]
    fn rejects_malformed_headers() {
        let error =
            parse_document("#SOUP \"/tmp/file.txt\"\na").expect_err("malformed header should fail");
        assert!(error.to_string().contains("malformed soup header"));
    }

    #[test]
    fn rejects_headers_missing_required_metadata() {
        let error = parse_document("#SOUP \"/tmp/file.txt\" #SOUPED_LINES 1\na")
            .expect_err("missing metadata should fail");
        assert!(error.to_string().contains("missing soup metadata"));
    }

    #[test]
    fn rejects_invalid_partial_ranges() {
        let error = parse_document(
            "#SOUP \"/tmp/file.txt\" #SOUP_PARTIAL_LINES 3-2 #SOUPED_LINES 1 #SOUP_TRAILING_NEWLINE 0\na",
        )
        .expect_err("invalid partial range should fail");
        assert!(error.to_string().contains("invalid partial soup range"));
    }

    #[test]
    fn preserves_empty_files() {
        let document =
            parse_document("#SOUP \"/tmp/empty.txt\" #SOUPED_LINES 0 #SOUP_TRAILING_NEWLINE 0")
                .expect("document should parse");

        assert!(document.blocks[0].content_lines.is_empty());
    }

    #[test]
    fn preserves_files_with_and_without_trailing_newline() {
        let with_newline = source_file("/tmp/with.txt", "hello\n");
        let without_newline = source_file("/tmp/without.txt", "hello");
        let document = serialize_document(&[with_newline, without_newline])
            .expect("document should serialize");
        let parsed = parse_document(&document).expect("document should parse");

        assert!(parsed.blocks[0].trailing_newline);
        assert!(!parsed.blocks[1].trailing_newline);
    }

    #[test]
    fn round_trips_document_exactly() {
        let files = vec![
            source_file("/tmp/empty.txt", ""),
            source_file("/tmp/multi.txt", "one\n\nthree\n"),
        ];

        let markdown = serialize_document(&files).expect("document should serialize");
        let parsed = parse_document(&markdown).expect("document should parse");

        assert_eq!(parsed.blocks.len(), 2);
        assert_eq!(parsed.blocks[0].logical_line_count, 0);
        assert_eq!(parsed.blocks[1].content_lines, vec!["one", "", "three"]);
        assert!(parsed.blocks[1].trailing_newline);
    }

    #[test]
    fn accepts_a_trailing_newline_after_the_last_block() {
        let document = parse_document("#SOUP \"/tmp/file.txt\" #SOUPED_LINES 1 #SOUP_TRAILING_NEWLINE true\nhello\n")
            .expect("document should parse");

        assert_eq!(document.blocks.len(), 1);
        assert_eq!(document.blocks[0].content_lines, vec!["hello"]);
        assert!(document.blocks[0].trailing_newline);
    }
}
