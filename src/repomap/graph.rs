use std::collections::{BTreeMap, BTreeSet, HashSet};

use super::tags::{Tag, TagKind};

pub struct RankedTag {
    pub rank: f64,
    pub tag: Tag,
}

pub fn build_and_rank(
    all_tags: &[Tag],
    chat_rel_fnames: &HashSet<String>,
    mentioned_idents: &HashSet<String>,
    mentioned_fnames: &HashSet<String>,
) -> Vec<RankedTag> {
    if all_tags.is_empty() {
        return Vec::new();
    }

    let mut defines: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut references: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut personalization: BTreeMap<String, f64> = BTreeMap::new();
    let mut all_rel_fnames: BTreeSet<String> = BTreeSet::new();

    for tag in all_tags {
        all_rel_fnames.insert(tag.rel_fname.clone());

        match tag.kind {
            TagKind::Def => {
                defines
                    .entry(tag.name.clone())
                    .or_default()
                    .insert(tag.rel_fname.clone());
            }
            TagKind::Ref => {
                references
                    .entry(tag.name.clone())
                    .or_default()
                    .insert(tag.rel_fname.clone());
            }
        }
    }

    for rel_fname in chat_rel_fnames {
        if all_rel_fnames.contains(rel_fname) {
            personalization.insert(rel_fname.clone(), 100.0);
        }
    }

    let mut edge_weights: BTreeMap<(String, String), f64> = BTreeMap::new();

    for (name, ref_fnames) in &references {
        if let Some(defs) = defines.get(name) {
            for ref_fname in ref_fnames {
                for def_fname in defs {
                    if ref_fname != def_fname {
                        *edge_weights
                            .entry((ref_fname.clone(), def_fname.clone()))
                            .or_insert(0.0) += 1.0;
                    }
                }
            }
        }
    }

    let ranks = pagerank(&edge_weights, &personalization, &all_rel_fnames);

    let mut ranked_tags = Vec::new();

    for tag in all_tags {
        if tag.kind != TagKind::Def {
            continue;
        }

        let file_rank = *ranks.get(&tag.rel_fname).unwrap_or(&0.0);

        let mut boost = 1.0;
        if mentioned_idents.contains(&tag.name) {
            boost *= 10.0;
        }
        if mentioned_fnames.contains(&tag.rel_fname) {
            boost *= 5.0;
        }
        if chat_rel_fnames.contains(&tag.rel_fname) {
            boost *= 20.0;
        }

        let final_rank = file_rank * boost;
        ranked_tags.push(RankedTag {
            rank: final_rank,
            tag: tag.clone(),
        });
    }

    ranked_tags.sort_by(|a, b| {
        b.rank
            .partial_cmp(&a.rank)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.tag.rel_fname.cmp(&b.tag.rel_fname))
            .then_with(|| a.tag.line.cmp(&b.tag.line))
            .then_with(|| a.tag.name.cmp(&b.tag.name))
    });
    ranked_tags
}

fn pagerank(
    edge_weights: &BTreeMap<(String, String), f64>,
    personalization: &BTreeMap<String, f64>,
    all_nodes: &BTreeSet<String>,
) -> BTreeMap<String, f64> {
    let n = all_nodes.len();
    if n == 0 {
        return BTreeMap::new();
    }

    let damping = 0.85_f64;
    let max_iter = 100;
    let tol = 1e-6_f64;

    let total_pers: f64 = personalization.values().sum();
    let pers_norm: BTreeMap<String, f64> = if total_pers > 0.0 {
        personalization
            .iter()
            .map(|(k, v)| (k.clone(), v / total_pers))
            .collect()
    } else {
        all_nodes
            .iter()
            .map(|node| (node.clone(), 1.0 / n as f64))
            .collect()
    };

    let uniform = 1.0 / n as f64;
    let mut ranks: BTreeMap<String, f64> = all_nodes
        .iter()
        .map(|node| (node.clone(), uniform))
        .collect();

    let mut out_strength: BTreeMap<String, f64> = BTreeMap::new();
    for node in all_nodes {
        let mut strength = 0.0;
        for ((src, _), w) in edge_weights {
            if src == node {
                strength += w;
            }
        }
        out_strength.insert(node.clone(), strength);
    }

    for _ in 0..max_iter {
        let mut new_ranks: BTreeMap<String, f64> = all_nodes
            .iter()
            .map(|node| {
                let p = pers_norm.get(node).copied().unwrap_or(0.0);
                (node.clone(), (1.0 - damping) * p)
            })
            .collect();

        for node in all_nodes {
            let rank = *ranks.get(node).unwrap_or(&0.0);
            let strength = *out_strength.get(node).unwrap_or(&0.0);

            if strength > 0.0 {
                for ((src, tgt), w) in edge_weights {
                    if src == node {
                        let contribution = damping * rank * w / strength;
                        *new_ranks.entry(tgt.clone()).or_insert(0.0) += contribution;
                    }
                }
            } else {
                for target in all_nodes {
                    let tp = pers_norm.get(target).copied().unwrap_or(0.0);
                    *new_ranks.entry(target.clone()).or_insert(0.0) +=
                        damping * rank * tp;
                }
            }
        }

        let mut diff = 0.0;
        for node in all_nodes {
            let old = *ranks.get(node).unwrap_or(&0.0);
            let newv = *new_ranks.get(node).unwrap_or(&0.0);
            diff += (newv - old).abs();
        }

        ranks = new_ranks;

        if diff < tol {
            break;
        }
    }

    ranks
}
