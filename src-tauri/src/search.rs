//! Campaign-wide search index.
//!
//! A flat in-memory index of every parseable blueprint: titles, tags,
//! statblock names and body text. The index is rebuilt lazily when the
//! dirty flag is set (campaign opened/switched, blueprint mutated, or the
//! notify watcher observed a file change) and queried with a mix of
//! explicit matches (substring/prefix/tag) and fuzzy title matching.

use crate::campaign::collect_md_files;
use crate::doc::{self, DocKind};
use fuzzy_matcher::skim::SkimMatcherV2;
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

const MAX_HITS: usize = 30;

#[derive(Debug, Clone)]
pub struct SearchDoc {
    pub rel_path: String,
    pub kind: DocKind,
    pub title: String,
    pub tags: Vec<String>,
    /// Lowercased body text for substring matching.
    pub text: String,
    pub statblock_name: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub rel_path: String,
    pub kind: DocKind,
    pub title: String,
    pub snippet: String,
    pub score: f64,
}

/// Parse the whole campaign into search docs. Unparseable files are skipped
/// (their raw text is still reachable through the file tree).
pub fn build(campaign_root: &Path) -> Result<Vec<SearchDoc>, String> {
    let mut docs = Vec::new();
    for path in collect_md_files(campaign_root, 0).map_err(|e| e.to_string())? {
        let rel = path
            .strip_prefix(campaign_root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();
        let Ok(raw) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(parsed) = doc::parse_doc(&rel, &raw) else {
            continue;
        };
        let (title, tags, statblock_name) = match parsed.kind {
            DocKind::Scene => (
                parsed.scene.as_ref().map(|m| m.title.clone()).unwrap_or_default(),
                parsed.scene.as_ref().map(|m| m.tags.clone()).unwrap_or_default(),
                None,
            ),
            DocKind::Encounter => (
                parsed.encounter.as_ref().map(|m| m.title.clone()).unwrap_or_default(),
                parsed.encounter.as_ref().map(|m| m.tags.clone()).unwrap_or_default(),
                None,
            ),
            DocKind::Statblock => (
                parsed.statblock.as_ref().map(|m| m.name.clone()).unwrap_or_default(),
                parsed.statblock.as_ref().map(|m| m.tags.clone()).unwrap_or_default(),
                parsed.statblock.as_ref().map(|m| m.name.clone()),
            ),
            DocKind::Item => (
                parsed.item.as_ref().map(|m| m.name.clone()).unwrap_or_default(),
                parsed.item.as_ref().map(|m| m.tags.clone()).unwrap_or_default(),
                None,
            ),
        };
        docs.push(SearchDoc {
            rel_path: rel,
            kind: parsed.kind,
            title,
            tags,
            text: parsed.body.to_lowercase(),
            statblock_name,
        });
    }
    Ok(docs)
}

fn kind_rank(kind: DocKind) -> u8 {
    match kind {
        DocKind::Scene => 0,
        DocKind::Encounter => 1,
        DocKind::Statblock => 2,
        DocKind::Item => 3,
    }
}

fn snippet(text: &str, token: &str) -> String {
    let Some(idx) = text.find(token) else {
        return String::new();
    };
    let start = idx.saturating_sub(30);
    let end = (idx + token.len() + 40).min(text.len());
    let mut s = text[start..end].trim().to_string();
    if start > 0 {
        s = format!("…{s}");
    }
    if end < text.len() {
        s.push('…');
    }
    s.replace('\n', " ")
}

pub fn run(docs: &[SearchDoc], query: &str) -> Vec<SearchHit> {
    let tokens: Vec<String> = query
        .split_whitespace()
        .flat_map(|t| t.split('/'))
        .filter(|t| !t.is_empty())
        .map(|t| t.to_lowercase())
        .collect();
    if tokens.is_empty() {
        return Vec::new();
    }
    let matcher = SkimMatcherV2::default();
    let mut hits = Vec::new();

    'doc: for d in docs {
        let title_l = d.title.to_lowercase();
        let name_l = d.statblock_name.as_deref().map(str::to_lowercase);
        let mut total = 0.0f64;
        let mut found_snippet: Option<String> = None;

        for tok in &tokens {
            let mut best = 0.0f64;

            if title_l.contains(tok) {
                best = best.max(if title_l.starts_with(tok) { 100.0 } else { 80.0 });
            }
            if let Some(name) = &name_l {
                if name.contains(tok) {
                    best = best.max(if name.starts_with(tok) { 95.0 } else { 75.0 });
                }
            }
            for tag in &d.tags {
                let tag_l = tag.to_lowercase();
                if tag_l == *tok {
                    best = best.max(70.0);
                } else if tag_l.starts_with(tok) {
                    best = best.max(55.0);
                } else if tag_l.contains(tok) {
                    best = best.max(45.0);
                }
            }
            if d.text.contains(tok) {
                best = best.max(20.0);
                if found_snippet.is_none() {
                    found_snippet = Some(snippet(&d.text, tok));
                }
            }
            // Fuzzy rescue: token found nowhere explicitly → try the title.
            if best == 0.0 {
                if let Some((score, _)) = matcher.fuzzy(&title_l, tok, true) {
                    best = score as f64 * 0.1;
                } else if let Some(name) = &name_l {
                    if let Some((score, _)) = matcher.fuzzy(name, tok, true) {
                        best = score as f64 * 0.09;
                    }
                }
            }
            if best <= 0.0 {
                continue 'doc;
            }
            total += best;
        }

        hits.push(SearchHit {
            rel_path: d.rel_path.clone(),
            kind: d.kind,
            title: d.title.clone(),
            snippet: found_snippet.unwrap_or_default(),
            score: total,
        });
    }

    hits.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then(kind_rank(a.kind).cmp(&kind_rank(b.kind)))
            .then(a.title.to_lowercase().cmp(&b.title.to_lowercase()))
    });
    hits.truncate(MAX_HITS);
    hits
}

/// Rebuild (if dirty) and query the shared index slot.
pub fn search(
    campaign_root: &Path,
    slot: &Mutex<Option<Vec<SearchDoc>>>,
    dirty: &AtomicBool,
    query: &str,
) -> Result<Vec<SearchHit>, String> {
    let mut guard = slot.lock().unwrap();
    if guard.is_none() || dirty.swap(false, Ordering::Relaxed) {
        *guard = Some(build(campaign_root)?);
    }
    let docs = guard.as_ref().unwrap();
    Ok(run(docs, query))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::campaign::slugify;
    use std::sync::Arc;

    const SCENE: &str = "---\ntype: scene\ntitle: The Prancing Pony\ntags: [tavern, social]\n---\nThe fire crackles as the door swings open. Volo raises a mug.\n";
    const GOBLIN: &str = "---\ntype: statblock\nname: Goblin\nkind: monster\nhp: 7\n---\nNimble and sneaky.\n";
    const AMBUSH: &str = "---\ntype: encounter\ntitle: Goblin Ambush\ntags: [forest]\ncombatants:\n  - name: Goblin A\n    hp: 7\n---\nSet an ambush along the road.\n";
    const ITEM: &str = "---\ntype: item\nname: Cloak of Embers\nrarity: uncommon\n---\nWarm to the touch.\n";

    fn setup() -> tempdir::TempDir {
        let dir = tempdir::TempDir::new("gmsearch").unwrap();
        for d in ["scenes", "encounters", "statblocks", "items"] {
            std::fs::create_dir_all(dir.path().join(d)).unwrap();
        }
        std::fs::write(dir.path().join("scenes/pony.md"), SCENE).unwrap();
        std::fs::write(dir.path().join("statblocks/goblin.md"), GOBLIN).unwrap();
        std::fs::write(dir.path().join("encounters/ambush.md"), AMBUSH).unwrap();
        std::fs::write(dir.path().join("items/cloak.md"), ITEM).unwrap();
        dir
    }

    // Minimal RAII temp dir.
    mod tempdir {
        use std::{
            fs,
            path::{Path, PathBuf},
            sync::atomic::{AtomicU64, Ordering},
        };
        static COUNTER: AtomicU64 = AtomicU64::new(0);
        pub struct TempDir(PathBuf);
        impl TempDir {
            pub fn new(prefix: &str) -> std::io::Result<TempDir> {
                let n = COUNTER.fetch_add(1, Ordering::SeqCst);
                let p = std::env::temp_dir().join(format!("{prefix}-{}-{n}", std::process::id()));
                fs::create_dir_all(&p)?;
                Ok(TempDir(p))
            }
            pub fn path(&self) -> &Path {
                &self.0
            }
        }
        impl Drop for TempDir {
            fn drop(&mut self) {
                let _ = fs::remove_dir_all(&self.0);
            }
        }
    }

    fn slot() -> (Mutex<Option<Vec<SearchDoc>>>, Arc<AtomicBool>) {
        (
            Mutex::new(None),
            Arc::new(AtomicBool::new(true)),
        )
    }

    #[test]
    fn builds_index_from_campaign() {
        let dir = setup();
        let docs = build(dir.path()).unwrap();
        assert_eq!(docs.len(), 4);
        assert!(docs.iter().any(|d| d.title == "The Prancing Pony" && d.kind == DocKind::Scene));
        assert!(docs.iter().any(|d| d.statblock_name == Some("Goblin".into())));
    }

    #[test]
    fn empty_query_matches_nothing() {
        assert!(run(&[], "").is_empty());
        let dir = setup();
        let docs = build(dir.path()).unwrap();
        assert!(run(&docs, "   ").is_empty());
    }

    #[test]
    fn finds_by_title_tag_body_and_name() {
        let dir = setup();
        let docs = build(dir.path()).unwrap();

        let hits = run(&docs, "prancing");
        assert_eq!(hits[0].title, "The Prancing Pony");

        let hits = run(&docs, "tavern");
        assert_eq!(hits[0].title, "The Prancing Pony"); // tag match outranks body

        let hits = run(&docs, "crackles");
        assert_eq!(hits[0].kind, DocKind::Scene);
        assert!(hits[0].snippet.contains("crackles"));

        let hits = run(&docs, "goblin");
        assert!(hits.iter().any(|h| h.kind == DocKind::Statblock));
        assert!(hits.iter().any(|h| h.kind == DocKind::Encounter));

        let hits = run(&docs, "embers");
        assert_eq!(hits[0].kind, DocKind::Item);
    }

    #[test]
    fn multi_token_requires_all() {
        let dir = setup();
        let docs = build(dir.path()).unwrap();
        let hits = run(&docs, "goblin ambush");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].kind, DocKind::Encounter);
        let hits = run(&docs, "goblin pony");
        assert!(hits.is_empty());
    }

    #[test]
    fn fuzzy_rescues_typos() {
        let dir = setup();
        let docs = build(dir.path()).unwrap();
        let hits = run(&docs, "gobln");
        assert!(hits.iter().any(|h| h.title == "Goblin"));
    }

    #[test]
    fn dirty_flag_triggers_rebuild() {
        let dir = setup();
        let (slot, dirty) = slot();
        let root = dir.path().to_path_buf();

        let hits = search(&root, &slot, &dirty, "volkaanix").unwrap();
        assert!(hits.is_empty());

        // External edit + dirty flag → next search sees it.
        let rel = format!("statblocks/{}.md", slugify("Volkani X"));
        std::fs::write(
            root.join(&rel),
            "---\ntype: statblock\nname: Volkani X\nkind: monster\nhp: 99\n---\n",
        )
        .unwrap();
        dirty.store(true, Ordering::Relaxed);
        let hits = search(&root, &slot, &dirty, "volkani").unwrap();
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].title, "Volkani X");
    }

    #[test]
    fn results_capped_and_sorted() {
        let dir = setup();
        let docs = build(dir.path()).unwrap();
        let hits = run(&docs, "the");
        assert!(hits.len() <= MAX_HITS);
        // Scores descend.
        for w in hits.windows(2) {
            assert!(w[0].score >= w[1].score);
        }
    }
}
