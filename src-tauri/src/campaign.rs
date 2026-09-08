use serde::Serialize;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

/// Aggregate file counts by conventional campaign folders.
#[derive(Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct CampaignCounts {
    pub scenes: usize,
    pub encounters: usize,
    pub npcs: usize,
    pub statblocks: usize,
    pub items: usize,
    pub total: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CampaignInfo {
    pub name: String,
    pub path: String,
    pub counts: CampaignCounts,
}

impl CampaignInfo {
    pub fn from_dir(dir: &Path) -> std::io::Result<CampaignInfo> {
        let counts = count_campaign(dir);
        let name = dir
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_else(|| dir.display().to_string());
        Ok(CampaignInfo {
            name,
            path: dir.display().to_string(),
            counts,
        })
    }
}

/// Count markdown files inside conventional campaign folders (non-recursive top level).
fn count_campaign(root: &Path) -> CampaignCounts {
    let mut counts = CampaignCounts::default();
    for entry in fs::read_dir(root).ok().into_iter().flatten().flatten() {
        if !entry.path().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        let slot = match name.as_str() {
            "scenes" => Some(&mut counts.scenes),
            "encounters" => Some(&mut counts.encounters),
            "npcs" => Some(&mut counts.npcs),
            "statblocks" | "monsters" => Some(&mut counts.statblocks),
            "items" => Some(&mut counts.items),
            _ => None,
        };
        if let Some(slot) = slot {
            *slot += fs::read_dir(entry.path())
                .map(|rd| {
                    rd.flatten()
                        .filter(|e| {
                            e.path()
                                .extension()
                                .map(|x| x == "md" || x == "json")
                                .unwrap_or(false)
                        })
                        .count()
                })
                .unwrap_or(0);
        }
    }
    counts.total = count_md_recursive(root, 0);
    counts
}

fn count_md_recursive(dir: &Path, depth: usize) -> usize {
    if depth > 3 {
        return 0;
    }
    let mut total = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if name.starts_with('.') || name == "node_modules" || name == "target" {
                continue;
            }
            if path.is_dir() {
                total += count_md_recursive(&path, depth + 1);
            } else if path.extension().map(|x| x == "md").unwrap_or(false) {
                total += 1;
            }
        }
    }
    total
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TreeNode {
    pub name: String,
    pub rel_path: String,
    pub kind: NodeKind,
    pub ext: Option<String>,
    pub open: bool,
    pub children: Vec<TreeNode>,
}

#[derive(Serialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum NodeKind {
    Dir,
    File,
}

const SKIP_DIRS: [&str; 5] = ["node_modules", "target", "dist", ".git", ".state"];
const INCLUDED_EXTS: [&str; 2] = ["md", "json"];
const MAX_DEPTH: usize = 4;

/// Filesystem-safe slug: lowercase, non-alphanumerics → `-`, trimmed.
/// Empty names become "combatant".
pub fn slugify(name: &str) -> String {
    let mut slug = String::new();
    let mut prev_dash = true;
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            prev_dash = false;
        } else if !prev_dash {
            slug.push('-');
            prev_dash = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        slug.push_str("combatant");
    }
    slug
}

/// Recursively collect markdown files under `dir`, skipping hidden/skip dirs.
pub fn collect_md_files(dir: &Path, depth: usize) -> std::io::Result<Vec<PathBuf>> {
    if depth > MAX_DEPTH {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in fs::read_dir(dir)?.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || SKIP_DIRS.contains(&name.as_str()) {
            continue;
        }
        if path.is_dir() {
            out.extend(collect_md_files(&path, depth + 1)?);
        } else if path
            .extension()
            .map(|x| x == "md")
            .unwrap_or(false)
        {
            out.push(path);
        }
    }
    Ok(out)
}

pub fn read_tree(root: &Path) -> std::io::Result<Vec<TreeNode>> {
    walk(root, root, 0)
}

fn walk(root: &Path, dir: &Path, depth: usize) -> std::io::Result<Vec<TreeNode>> {
    if depth > MAX_DEPTH {
        return Ok(Vec::new());
    }
    let mut dirs: BTreeMap<String, PathBuf> = BTreeMap::new();
    let mut files: BTreeMap<String, PathBuf> = BTreeMap::new();

    for entry in fs::read_dir(dir)?.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.') || SKIP_DIRS.contains(&name.as_str()) {
            continue;
        }
        if path.is_dir() {
            dirs.insert(name.to_lowercase(), path);
        } else if path
            .extension()
            .map(|x| INCLUDED_EXTS.contains(&x.to_string_lossy().to_lowercase().as_str()))
            .unwrap_or(false)
        {
            files.insert(name.to_lowercase(), path);
        }
    }

    let mut nodes = Vec::new();
    for (_, path) in dirs {
        let name = path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let rel_path = rel(root, &path);
        let children = walk(root, &path, depth + 1)?;
        // Empty directories carry no value in the library tree.
        if children.is_empty() {
            continue;
        }
        nodes.push(TreeNode {
            name,
            rel_path,
            kind: NodeKind::Dir,
            ext: None,
            // Top-level folders start expanded.
            open: depth == 0,
            children,
        });
    }
    for (_, path) in files {
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        let ext = path
            .extension()
            .map(|x| x.to_string_lossy().to_string());
        nodes.push(TreeNode {
            name,
            rel_path: rel(root, &path),
            kind: NodeKind::File,
            ext,
            open: false,
            children: Vec::new(),
        });
    }
    Ok(nodes)
}

fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}
