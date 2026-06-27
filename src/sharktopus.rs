use std::path::PathBuf;
use std::process::Command;

use crate::config::Config;
use crate::error::SoupifyError;
use crate::pathing::expand_tilde;

const MOVE_RULE_NAME: &str = "move soupified files";
const AUTO_DESOUPIFY_RULE_NAME: &str = "auto-desoupify";
const SOUP_PATTERN: &str = "*.soup.md";

pub fn is_available() -> bool {
    Command::new("sharktopus")
        .arg("--help")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn ensure_rules(config: &Config) -> Result<Vec<String>, SoupifyError> {
    if !is_available() {
        return Err(SoupifyError::ConfigError(
            "sharktopus is not available on PATH".to_string(),
        ));
    }

    let mut messages = Vec::new();
    let existing_rules = list_rules()?;

    let to_desoupify = config
        .to_desoupify_folder
        .as_deref()
        .map(expand_tilde)
        .or_else(crate::config::default_to_desoupify_folder)
        .unwrap_or_else(|| {
            let home = std::env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("~"));
            home.join(".soupify").join("to_desoupify")
        });

    if !has_rule_named(&existing_rules, MOVE_RULE_NAME) {
        add_move_rule(&to_desoupify)?;
        messages.push(format!(
            "added Sharktopus rule '{}' -> {}",
            MOVE_RULE_NAME,
            to_desoupify.display()
        ));
    } else {
        messages.push(format!(
            "Sharktopus rule '{}' already exists",
            MOVE_RULE_NAME
        ));
    }

    if config.auto_desoupify {
        if !has_rule_named(&existing_rules, AUTO_DESOUPIFY_RULE_NAME) {
            add_auto_desoupify_rule()?;
            messages.push(format!(
                "added Sharktopus rule '{}'",
                AUTO_DESOUPIFY_RULE_NAME
            ));
        } else {
            messages.push(format!(
                "Sharktopus rule '{}' already exists",
                AUTO_DESOUPIFY_RULE_NAME
            ));
        }
    }

    Ok(messages)
}

fn list_rules() -> Result<String, SoupifyError> {
    let output = Command::new("sharktopus")
        .args(["list-rules", "--pattern", SOUP_PATTERN])
        .output()
        .map_err(|error| {
            SoupifyError::ConfigError(format!("failed to run sharktopus list-rules: {error}"))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SoupifyError::ConfigError(format!(
            "sharktopus list-rules failed: {stderr}"
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn has_rule_named(rules_output: &str, name: &str) -> bool {
    rules_output.contains(name)
}

fn add_move_rule(destination: &PathBuf) -> Result<(), SoupifyError> {
    let dest_str = destination.to_string_lossy().to_string();
    let output = Command::new("sharktopus")
        .args([
            "add-rule",
            "--name",
            MOVE_RULE_NAME,
            "--pattern",
            SOUP_PATTERN,
            "--action",
            "move",
            "--destination",
            &dest_str,
        ])
        .output()
        .map_err(|error| {
            SoupifyError::ConfigError(format!("failed to run sharktopus add-rule: {error}"))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SoupifyError::ConfigError(format!(
            "sharktopus add-rule (move) failed: {stderr}"
        )));
    }

    Ok(())
}

fn add_auto_desoupify_rule() -> Result<(), SoupifyError> {
    let output = Command::new("sharktopus")
        .args([
            "add-rule",
            "--name",
            AUTO_DESOUPIFY_RULE_NAME,
            "--pattern",
            SOUP_PATTERN,
            "--action",
            "run",
            "--command",
            "soupify -d __FILE__",
        ])
        .output()
        .map_err(|error| {
            SoupifyError::ConfigError(format!("failed to run sharktopus add-rule: {error}"))
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(SoupifyError::ConfigError(format!(
            "sharktopus add-rule (auto-desoupify) failed: {stderr}"
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn has_rule_named_detects_existing_rule() {
        let output = "08C81EF5   move soupified files           *.soup.md            move Yes      ~/dev/output/soupified\n";
        assert!(has_rule_named(output, "move soupified files"));
        assert!(!has_rule_named(output, "auto-desoupify"));
    }

    #[test]
    fn has_rule_named_matches_new_format() {
        let output = "08C81EF5   cli      move soupified files                     *.soup.md                    move     Yes      1     -> ~/.soupify/to_desoupify\n";
        assert!(has_rule_named(output, "move soupified files"));
        assert!(!has_rule_named(output, "auto-desoupify"));
    }

    #[test]
    fn has_rule_named_handles_empty_output() {
        assert!(!has_rule_named("", "any rule"));
    }

    #[test]
    fn constants_have_expected_values() {
        assert_eq!(MOVE_RULE_NAME, "move soupified files");
        assert_eq!(AUTO_DESOUPIFY_RULE_NAME, "auto-desoupify");
        assert_eq!(SOUP_PATTERN, "*.soup.md");
    }
}
