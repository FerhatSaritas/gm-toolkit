//! Statblock link resolution.
//!
//! A combatant's `statblock` field is either a campaign-relative (or
//! blueprint-relative) file path, or a bare creature name to look up across
//! the campaign. Resolution never escapes the campaign root: relative paths
//! are normalized lexically *before* joining, and everything is re-validated
//! with `ensure_in_campaign`.

use crate::doc::{self, Doc, DocKind};
use crate::ensure_in_campaign;
use std::path::{Component, Path, PathBuf};

/// Collapse `a/../b` and `./` segments lexically. `..` beyond the root is a
/// no-op, so the result can never climb out of the campaign folder.
fn normalize_rel(p: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for comp in p.components() {
        match comp {
            Component::Normal(c) => out.push(c),
            Component::ParentDir => {
                out.pop();
            }
            _ => {}
        }
    }
    out
}

fn looks_like_path(link: &str) -> bool {
    link.contains('/') || link.contains('\\') || link.ends_with(".md") || link.ends_with(".json")
}

fn load_doc_at(campaign_root: &Path, rel: &Path) -> Result<Doc, String> {
    let rel_str = rel.to_string_lossy().to_string();
    ensure_in_campaign(campaign_root, &rel_str)?;
    let raw = std::fs::read_to_string(campaign_root.join(&rel))
        .map_err(|e| format!("Cannot read statblock: {e}"))?;
    doc::parse_doc(&rel_str, &raw)
}

fn find_by_name(campaign_root: &Path, name: &str) -> Result<Doc, String> {
    // Walk conventional folders first, then everything, case-insensitive.
    let preferred = ["statblocks", "monsters", "npcs"];
    let mut bases: Vec<PathBuf> = preferred.iter().map(PathBuf::from).collect();
    bases.push(PathBuf::new());

    for base in bases {
        let root_dir = campaign_root.join(&base);
        if let Ok(files) = crate::campaign::collect_md_files(&root_dir, 0) {
            for path in files {
                let rel = path.strip_prefix(campaign_root).unwrap_or(&path).to_path_buf();
                if let Ok(parsed) = load_doc_at(campaign_root, &rel) {
                    if parsed.kind == DocKind::Statblock {
                        if let Some(meta) = &parsed.statblock {
                            if meta.name.eq_ignore_ascii_case(name) {
                                return Ok(parsed);
                            }
                        }
                    }
                }
            }
        }
    }
    Err(format!("No statblock named “{name}” found in the campaign"))
}

/// Open the statblock referenced by `link`, relative to `base_path`
/// (the blueprint that contains the reference), or by creature name.
pub fn open_at(
    campaign_root: &Path,
    base_path: Option<&str>,
    link: &str,
) -> Result<Doc, String> {
    let link = link.trim();
    if link.is_empty() {
        return Err("Empty statblock reference".to_string());
    }
    if !looks_like_path(link) {
        return find_by_name(campaign_root, link);
    }
    let base_dir = match base_path {
        Some(b) => Path::new(b)
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .to_path_buf(),
        None => PathBuf::new(),
    };
    let rel = normalize_rel(&base_dir.join(link));
    if rel.as_os_str().is_empty() {
        return Err(format!("Statblock reference “{link}” resolves outside the campaign"));
    }
    load_doc_at(campaign_root, &rel)
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOBLIN: &str = "---\ntype: statblock\nname: Goblin\nkind: monster\nhp: 7\nac: 15\nactions:\n  - name: Scimitar\n    text: \"Melee: +4 to hit, 1d6+2 slashing.\"\n---\nNotes.\n";
    const GRASHNAK: &str = "---\ntype: statblock\nname: Grashnak\nkind: npc\nhp: 45\nac: 15\n---\n";

    fn setup() -> tempdir::TempDir {
        let dir = tempdir::TempDir::new("gmstat").unwrap();
        std::fs::create_dir_all(dir.path().join("encounters")).unwrap();
        std::fs::create_dir_all(dir.path().join("npcs")).unwrap();
        std::fs::create_dir_all(dir.path().join("statblocks")).unwrap();
        std::fs::write(dir.path().join("statblocks/goblin.md"), GOBLIN).unwrap();
        std::fs::write(dir.path().join("npcs/grashnak.md"), GRASHNAK).unwrap();
        dir
    }

    // Minimal RAII temp dir (same pattern as encounter tests).
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

    #[test]
    fn normalizes_parent_refs() {
        assert_eq!(normalize_rel(Path::new("encounters/../npcs/g.md")), PathBuf::from("npcs/g.md"));
        assert_eq!(normalize_rel(Path::new("../../npcs/g.md")), PathBuf::from("npcs/g.md"));
        assert_eq!(normalize_rel(Path::new("./a/./b.md")), PathBuf::from("a/b.md"));
        assert_eq!(normalize_rel(Path::new("../../../../x.md")), PathBuf::from("x.md"));
    }

    #[test]
    fn resolves_relative_link_from_blueprint() {
        let dir = setup();
        let doc = open_at(dir.path(), Some("encounters/ambush.md"), "../npcs/grashnak.md").unwrap();
        assert_eq!(doc.statblock.unwrap().name, "Grashnak");
    }

    #[test]
    fn resolves_bare_name_case_insensitive() {
        let dir = setup();
        let doc = open_at(dir.path(), None, "GOBLIN").unwrap();
        assert_eq!(doc.statblock.unwrap().name, "Goblin");
    }

    #[test]
    fn escaped_links_stay_inside_campaign() {
        let dir = setup();
        // Normalization collapses the climb; the file simply won't exist.
        assert!(open_at(dir.path(), Some("encounters/ambush.md"), "../../../etc/passwd.md").is_err());
        assert!(open_at(dir.path(), Some("encounters/ambush.md"), "..").is_err());
    }

    #[test]
    fn unknown_name_is_an_error() {
        let dir = setup();
        assert!(open_at(dir.path(), None, "Ancient Red Dragon").is_err());
    }
}
