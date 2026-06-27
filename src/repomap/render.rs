use std::collections::BTreeMap;
use std::path::Path;

use super::graph::RankedTag;

pub fn render_tree(ranked_tags: &[RankedTag], root: &Path) -> String {
    if ranked_tags.is_empty() {
        return String::new();
    }

    let mut file_tags: BTreeMap<String, Vec<(f64, &RankedTag)>> = BTreeMap::new();
    for rt in ranked_tags {
        file_tags
            .entry(rt.tag.rel_fname.clone())
            .or_default()
            .push((rt.rank, rt));
    }

    let mut sorted_files: Vec<(String, Vec<(f64, &RankedTag)>)> = file_tags.into_iter().collect();
    sorted_files.sort_by(|a, b| {
        let max_a = a.1.iter().map(|(r, _)| *r).fold(0.0_f64, |acc, v| acc.max(v));
        let max_b = b.1.iter().map(|(r, _)| *r).fold(0.0_f64, |acc, v| acc.max(v));
        max_b
            .partial_cmp(&max_a)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });

    let mut parts = Vec::new();

    for (rel_fname, tag_list) in &sorted_files {
        let abs_fname = root.join(rel_fname);
        let code = std::fs::read_to_string(&abs_fname).unwrap_or_default();

        let lines: Vec<&str> = code.lines().collect();

        let mut lois: Vec<usize> = tag_list
            .iter()
            .map(|(_, rt)| rt.tag.line)
            .collect();
        lois.sort();
        lois.dedup();

        let max_rank = tag_list
            .iter()
            .map(|(r, _)| *r)
            .fold(0.0_f64, |acc, v| acc.max(v));

        let mut out = format!("{}\n(Rank value: {:.4})\n\n", rel_fname, max_rank);

        for loi in &lois {
            if *loi >= 1 && *loi <= lines.len() {
                out.push_str(&format!("{:>4}: {}\n", loi, lines[loi - 1]));
            }
        }

        parts.push(out);
    }

    parts.join("\n")
}

pub fn count_tokens(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }

    let len = text.len();
    if len < 200 {
        return estimate_tokens(text);
    }

    let lines: Vec<&str> = text.lines().collect();
    let num_lines = lines.len();

    let step = (num_lines / 100).max(1);
    let sampled: Vec<&str> = lines.iter().step_by(step).copied().collect();
    let sample_text = sampled.join("\n");

    if sample_text.is_empty() {
        return estimate_tokens(text);
    }

    let sample_tokens = estimate_tokens(&sample_text);
    ((sample_tokens as f64 / sample_text.len() as f64) * len as f64) as usize
}

fn estimate_tokens(text: &str) -> usize {
    use std::sync::OnceLock;
    static TOKENIZER: OnceLock<Option<tiktoken_rs::CoreBPE>> = OnceLock::new();

    let tokenizer = TOKENIZER.get_or_init(|| {
        tiktoken_rs::get_bpe_from_tokenizer(tiktoken_rs::tokenizer::Tokenizer::O200kBase)
            .or_else(|_| tiktoken_rs::get_bpe_from_tokenizer(tiktoken_rs::tokenizer::Tokenizer::Cl100kBase))
            .ok()
    });

    if let Some(bpe) = tokenizer {
        let tokens = bpe.encode_with_special_tokens(text);
        return tokens.len();
    }

    (text.len() / 4).max(1)
}
