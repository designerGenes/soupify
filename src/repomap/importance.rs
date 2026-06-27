#![allow(dead_code)]

use std::collections::HashSet;

const IMPORTANT_FILENAMES: &[&str] = &[
    "README.md", "README.txt", "readme.md", "README.rst", "README",
    "requirements.txt", "Pipfile", "pyproject.toml", "setup.py", "setup.cfg",
    "package.json", "yarn.lock", "package-lock.json", "npm-shrinkwrap.json",
    "Dockerfile", "docker-compose.yml", "docker-compose.yaml",
    ".gitignore", ".gitattributes", ".dockerignore",
    "Makefile", "makefile", "CMakeLists.txt",
    "LICENSE", "LICENSE.txt", "LICENSE.md", "COPYING",
    "CHANGELOG.md", "CHANGELOG.txt", "HISTORY.md",
    "CONTRIBUTING.md", "CODE_OF_CONDUCT.md",
    ".env", ".env.example", ".env.local",
    "tox.ini", "pytest.ini", ".pytest.ini",
    ".flake8", ".pylintrc", "mypy.ini",
    "go.mod", "go.sum", "Cargo.toml", "Cargo.lock",
    "pom.xml", "build.gradle", "build.gradle.kts",
    "composer.json", "composer.lock",
    "Gemfile", "Gemfile.lock",
];

pub fn is_important(rel_path: &str) -> bool {
    let normalized = std::path::Path::new(rel_path);
    let file_name = normalized
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    let dir_name = normalized
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or("");

    if dir_name == ".github/workflows" && file_name.ends_with(".yml") || file_name.ends_with(".yaml") {
        return true;
    }
    if dir_name == ".github" && (file_name.ends_with(".md") || file_name.ends_with(".yml") || file_name.ends_with(".yaml")) {
        return true;
    }
    if dir_name == "docs" && (file_name.ends_with(".md") || file_name.ends_with(".rst") || file_name.ends_with(".txt")) {
        return true;
    }

    IMPORTANT_FILENAMES.contains(&file_name)
}

pub fn filter_important_files(rel_paths: &[String]) -> HashSet<String> {
    rel_paths
        .iter()
        .filter(|p| is_important(p))
        .cloned()
        .collect()
}
