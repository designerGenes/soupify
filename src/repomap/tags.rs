use std::path::Path;

use tree_sitter::{Language, Parser, Query, QueryCursor};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TagKind {
    Def,
    Ref,
}

#[derive(Debug, Clone)]
pub struct Tag {
    pub rel_fname: String,
    pub line: usize,
    pub name: String,
    pub kind: TagKind,
}

struct LangEntry {
    language: Language,
    query_src: &'static str,
}

fn lang_entry(path: &Path) -> Option<LangEntry> {
    let ext = path.extension()?.to_str()?.to_lowercase();
    let entry = match ext.as_str() {
        "rs" => LangEntry {
            language: tree_sitter_rust::language(),
            query_src: tree_sitter_rust::TAGS_QUERY,
        },
        "py" => LangEntry {
            language: tree_sitter_python::language(),
            query_src: tree_sitter_python::TAGS_QUERY,
        },
        "js" | "jsx" | "mjs" | "cjs" => LangEntry {
            language: tree_sitter_javascript::language(),
            query_src: tree_sitter_javascript::TAGS_QUERY,
        },
        "ts" => LangEntry {
            language: tree_sitter_typescript::language_typescript(),
            query_src: tree_sitter_typescript::TAGS_QUERY,
        },
        "tsx" => LangEntry {
            language: tree_sitter_typescript::language_tsx(),
            query_src: tree_sitter_typescript::TAGS_QUERY,
        },
        "go" => LangEntry {
            language: tree_sitter_go::language(),
            query_src: tree_sitter_go::TAGS_QUERY,
        },
        "c" | "h" => LangEntry {
            language: tree_sitter_c::language(),
            query_src: tree_sitter_c::TAGS_QUERY,
        },
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => LangEntry {
            language: tree_sitter_cpp::language(),
            query_src: tree_sitter_cpp::TAGS_QUERY,
        },
        "java" => LangEntry {
            language: tree_sitter_java::language(),
            query_src: tree_sitter_java::TAGS_QUERY,
        },
        "rb" => LangEntry {
            language: tree_sitter_ruby::language(),
            query_src: tree_sitter_ruby::TAGS_QUERY,
        },
        _ => return None,
    };
    Some(entry)
}

pub fn extract_tags(fname: &str, rel_fname: &str) -> Vec<Tag> {
    let path = Path::new(fname);
    let entry = match lang_entry(path) {
        Some(e) => e,
        None => return Vec::new(),
    };

    let source = match std::fs::read_to_string(fname) {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };

    let mut parser = Parser::new();
    if parser.set_language(&entry.language).is_err() {
        return Vec::new();
    }

    let tree = match parser.parse(source.as_bytes(), None) {
        Some(t) => t,
        None => return Vec::new(),
    };

    let query = match Query::new(&entry.language, entry.query_src) {
        Ok(q) => q,
        Err(_) => return Vec::new(),
    };

    let capture_names = query.capture_names();

    let mut cursor = QueryCursor::new();
    let matches = cursor.matches(&query, tree.root_node(), source.as_bytes());

    let mut tags = Vec::new();

    for m in matches {
        let mut kind = None;
        let mut name_node = None;

        for cap in m.captures {
            let cap_name = capture_names
                .get(cap.index as usize)
                .copied()
                .unwrap_or("");

            if cap_name.starts_with("definition.") {
                kind = Some(TagKind::Def);
            } else if cap_name.starts_with("reference.") {
                kind = Some(TagKind::Ref);
            } else if cap_name == "name" {
                name_node = Some(cap.node);
            }
        }

        if let (Some(kind), Some(node)) = (kind, name_node) {
            let line = node.start_position().row + 1;
            let name = node
                .utf8_text(source.as_bytes())
                .unwrap_or("")
                .to_string();

        tags.push(Tag {
            rel_fname: rel_fname.to_string(),
            line,
            name,
            kind,
        });
        }
    }

    tags
}
