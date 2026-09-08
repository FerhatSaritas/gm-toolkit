//! Catalog operations: listing creatures/encounters, creating blueprints,
//! adding combatants to encounters (blueprint *and* live state), duplicating
//! and deleting files. All writes go through the doc renderer so frontmatter
//! stays normalized.

use crate::campaign::{collect_md_files, slugify};
use crate::doc::{self, Combatant, DocKind, EncounterMeta, StatblockKind, StatblockMeta};
use crate::encounter::{self, CombatantState, EncounterState};
use crate::ensure_in_campaign;
use serde::Serialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatblockSummary {
    pub rel_path: String,
    pub name: String,
    pub kind: StatblockKind,
    pub hp: Option<i64>,
    pub ac: Option<i64>,
    pub initiative: Option<i64>,
    pub speed: Option<String>,
    pub power: Option<u8>,
    pub tags: Vec<String>,
    pub traits: usize,
    pub actions: usize,
    pub reactions: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EncounterSummary {
    pub rel_path: String,
    pub title: String,
    pub combatants: usize,
    pub tags: Vec<String>,
}

// ---------------------------------------------------------------------------
// Listing
// ---------------------------------------------------------------------------

pub fn list_statblocks(campaign_root: &Path) -> Result<Vec<StatblockSummary>, String> {
    let mut out = Vec::new();
    for path in collect_md_files(campaign_root, 0).map_err(|e| e.to_string())? {
        let rel = path
            .strip_prefix(campaign_root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();
        if let Ok(parsed) = load(campaign_root, &rel) {
            if parsed.kind != DocKind::Statblock {
                continue;
            }
            if let Some(m) = parsed.statblock {
                out.push(StatblockSummary {
                    rel_path: rel,
                    name: m.name,
                    kind: m.kind,
                    hp: m.hp,
                    ac: m.ac,
                    initiative: m.initiative,
                    speed: m.speed,
                    power: m.power,
                    tags: m.tags,
                    traits: m.traits.len(),
                    actions: m.actions.len(),
                    reactions: m.reactions.len(),
                });
            }
        }
    }
    out.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    Ok(out)
}

pub fn list_encounters(campaign_root: &Path) -> Result<Vec<EncounterSummary>, String> {
    let mut out = Vec::new();
    for path in collect_md_files(campaign_root, 0).map_err(|e| e.to_string())? {
        let rel = path
            .strip_prefix(campaign_root)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();
        if let Ok(parsed) = load(campaign_root, &rel) {
            if parsed.kind != DocKind::Encounter {
                continue;
            }
            if let Some(m) = parsed.encounter {
                out.push(EncounterSummary {
                    rel_path: rel,
                    title: m.title,
                    combatants: m.combatants.len(),
                    tags: m.tags,
                });
            }
        }
    }
    out.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
    Ok(out)
}

fn load(campaign_root: &Path, rel_path: &str) -> Result<doc::Doc, String> {
    let raw = fs::read_to_string(campaign_root.join(rel_path)).map_err(|e| e.to_string())?;
    doc::parse_doc(rel_path, &raw)
}

// ---------------------------------------------------------------------------
// Add combatant to encounter
// ---------------------------------------------------------------------------

/// Append a combatant to the blueprint; if a live state exists for that
/// encounter, the creature joins the running fight as well.
pub fn add_combatant_to_encounter(
    campaign_root: &Path,
    encounter_path: &str,
    combatant: &Combatant,
) -> Result<EncounterSummary, String> {
    ensure_in_campaign(campaign_root, encounter_path)?;
    let raw = fs::read_to_string(campaign_root.join(encounter_path)).map_err(|e| e.to_string())?;
    let mut doc = doc::parse_doc(encounter_path, &raw)?;
    let meta = doc
        .encounter
        .as_mut()
        .ok_or_else(|| "Not an encounter blueprint".to_string())?;
    if combatant.name.trim().is_empty() {
        return Err("Combatant needs a name".to_string());
    }
    meta.combatants.push(combatant.clone());
    let rendered = doc::render_doc(meta, &doc.body)?;
    fs::write(campaign_root.join(encounter_path), rendered).map_err(|e| e.to_string())?;

    // Join the live fight, if any.
    if let Ok(state) = load_state(campaign_root, encounter_path) {
        let mut state = state;
        let mut used: Vec<String> = state.combatants.iter().map(|c| c.id.clone()).collect();
        let hp = combatant.hp.unwrap_or(0);
        state.combatants.push(CombatantState {
            id: unique_id(&slugify(&combatant.name), &mut used),
            name: combatant.name.clone(),
            kind: combatant.kind,
            initiative: None,
            initiative_mod: combatant.initiative,
            hp,
            max_hp: combatant.hp,
            temp_hp: 0,
            ac: combatant.ac,
            statblock: combatant.statblock.clone(),
            conditions: Vec::new(),
        });
        encounter::save_state_at(campaign_root, &state)?;
    }

    Ok(EncounterSummary {
        rel_path: encounter_path.to_string(),
        title: meta.title.clone(),
        combatants: meta.combatants.len(),
        tags: meta.tags.clone(),
    })
}

fn load_state(campaign_root: &Path, encounter_path: &str) -> Result<EncounterState, String> {
    let path = state_file(campaign_root, encounter_path);
    if !path.is_file() {
        return Err("No live state".to_string());
    }
    let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    serde_json::from_str(&raw).map_err(|e| e.to_string())
}

fn state_file(campaign_root: &Path, encounter_path: &str) -> PathBuf {
    let rel = Path::new(encounter_path);
    let parent = rel.parent().unwrap_or_else(|| Path::new(""));
    let stem = rel
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "encounter".to_string());
    campaign_root
        .join(".state")
        .join(parent)
        .join(format!("{stem}.json"))
}

fn unique_id(base: &str, used: &mut Vec<String>) -> String {
    let mut candidate = base.to_string();
    let mut n = 2;
    while used.iter().any(|u| u == &candidate) {
        candidate = format!("{base}-{n}");
        n += 1;
    }
    used.push(candidate.clone());
    candidate
}

// ---------------------------------------------------------------------------
// Creation
// ---------------------------------------------------------------------------

fn unique_file_in(folder: &Path, base: &str) -> PathBuf {
    let mut candidate = folder.join(format!("{base}.md"));
    let mut n = 2;
    while candidate.exists() {
        candidate = folder.join(format!("{base}-{n}.md"));
        n += 1;
    }
    candidate
}

pub fn create_encounter(
    campaign_root: &Path,
    title: &str,
    combatant: Option<&Combatant>,
) -> Result<String, String> {
    let title = title.trim();
    if title.is_empty() {
        return Err("Encounter needs a title".to_string());
    }
    let folder = campaign_root.join("encounters");
    fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
    let path = unique_file_in(&folder, &slugify(title));
    let rel = path
        .strip_prefix(campaign_root)
        .unwrap_or(&path)
        .to_string_lossy()
        .to_string();

    let meta = EncounterMeta {
        doc_type: "encounter".to_string(),
        title: title.to_string(),
        combatants: combatant.cloned().into_iter().collect(),
        ..Default::default()
    };
    let body = "# Setup\n\n## Tactics\n";
    let rendered = doc::render_doc(&meta, body)?;
    fs::write(&path, rendered).map_err(|e| e.to_string())?;
    Ok(rel)
}

pub fn create_statblock(
    campaign_root: &Path,
    name: &str,
    kind: StatblockKind,
    hp: Option<i64>,
    ac: Option<i64>,
    power: Option<u8>,
) -> Result<String, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("Creature needs a name".to_string());
    }
    let meta = StatblockMeta {
        doc_type: "statblock".to_string(),
        name: name.to_string(),
        kind,
        hp,
        ac,
        power,
        ..Default::default()
    };
    let body = "## Actions\n\n- **Attack.** 1d6+0 damage.\n";
    write_statblock(campaign_root, &meta, body)
}

/// Write a full statblock blueprint (used by creation and import).
/// Targets the conventional folder for its kind; filename is uniquified.
pub fn write_statblock(
    campaign_root: &Path,
    meta: &StatblockMeta,
    body: &str,
) -> Result<String, String> {
    let folder = match meta.kind {
        StatblockKind::Npc => "npcs",
        StatblockKind::Player => "party",
        StatblockKind::Monster => "statblocks",
    };
    let folder_abs = campaign_root.join(folder);
    fs::create_dir_all(&folder_abs).map_err(|e| e.to_string())?;
    let path = unique_file_in(&folder_abs, &slugify(&meta.name));
    let rel = path
        .strip_prefix(campaign_root)
        .unwrap_or(&path)
        .to_string_lossy()
        .to_string();
    let rendered = doc::render_doc(meta, body)?;
    fs::write(&path, rendered).map_err(|e| e.to_string())?;
    Ok(rel)
}

/// Write an item blueprint into `items/`.
pub fn write_item(campaign_root: &Path, meta: &crate::doc::ItemMeta, body: &str) -> Result<String, String> {
    let folder_abs = campaign_root.join("items");
    fs::create_dir_all(&folder_abs).map_err(|e| e.to_string())?;
    let path = unique_file_in(&folder_abs, &slugify(&meta.name));
    let rel = path
        .strip_prefix(campaign_root)
        .unwrap_or(&path)
        .to_string_lossy()
        .to_string();
    let normalized = crate::doc::ItemMeta {
        doc_type: "item".into(),
        ..meta.clone()
    };
    let rendered = doc::render_doc(&normalized, body)?;
    fs::write(&path, rendered).map_err(|e| e.to_string())?;
    Ok(rel)
}

/// Write an encounter blueprint into `encounters/`.
pub fn write_encounter(
    campaign_root: &Path,
    meta: &EncounterMeta,
    body: &str,
) -> Result<String, String> {
    let folder_abs = campaign_root.join("encounters");
    fs::create_dir_all(&folder_abs).map_err(|e| e.to_string())?;
    let path = unique_file_in(&folder_abs, &slugify(&meta.title));
    let rel = path
        .strip_prefix(campaign_root)
        .unwrap_or(&path)
        .to_string_lossy()
        .to_string();
    let rendered = doc::render_doc(meta, body)?;
    fs::write(&path, rendered).map_err(|e| e.to_string())?;
    Ok(rel)
}

// ---------------------------------------------------------------------------
// Duplicate & delete
// ---------------------------------------------------------------------------

pub fn duplicate_file(campaign_root: &Path, rel_path: &str) -> Result<String, String> {
    let parsed = load(campaign_root, rel_path)?;
    let src = Path::new(rel_path);
    let folder = src.parent().unwrap_or_else(|| Path::new(""));
    let stem = src
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "copy".to_string());
    let folder_abs = campaign_root.join(folder);
    let dest = unique_file_in(&folder_abs, &format!("{stem}-copy"));
    let dest_rel = dest
        .strip_prefix(campaign_root)
        .unwrap_or(&dest)
        .to_string_lossy()
        .to_string();

    let body = parsed.body.clone();
    let rendered = match parsed.kind {
        DocKind::Scene => {
            let mut m = parsed.scene.clone().unwrap_or_default();
            m.doc_type = "scene".into();
            m.title = format!("{} (Copy)", m.title);
            doc::render_doc(&m, &body)?
        }
        DocKind::Encounter => {
            let mut m = parsed.encounter.clone().unwrap_or_default();
            m.doc_type = "encounter".into();
            m.title = format!("{} (Copy)", m.title);
            doc::render_doc(&m, &body)?
        }
        DocKind::Statblock => {
            let mut m = parsed.statblock.clone().unwrap_or_default();
            m.doc_type = "statblock".into();
            m.name = format!("{} (Copy)", m.name);
            doc::render_doc(&m, &body)?
        }
        DocKind::Item => {
            let mut m = parsed.item.clone().unwrap_or_default();
            m.doc_type = "item".into();
            m.name = format!("{} (Copy)", m.name);
            doc::render_doc(&m, &body)?
        }
    };
    fs::write(&dest, rendered).map_err(|e| e.to_string())?;
    Ok(dest_rel)
}

pub fn delete_file(campaign_root: &Path, rel_path: &str) -> Result<(), String> {
    ensure_in_campaign(campaign_root, rel_path)?;
    fs::remove_file(campaign_root.join(rel_path)).map_err(|e| e.to_string())?;
    // Drop any orphaned live state for deleted encounters.
    let state = state_file(campaign_root, rel_path);
    if state.is_file() {
        let _ = fs::remove_file(state);
    }
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::CombatantKind;

    const GOBLIN: &str = "---\ntype: statblock\nname: Goblin\nkind: monster\nhp: 7\nac: 15\npower: 1\nactions:\n  - name: Scimitar\n    text: \"1d6+2 slashing.\"\n---\n";

    fn setup() -> tempdir::TempDir {
        let dir = tempdir::TempDir::new("gmcat").unwrap();
        for d in ["encounters", "statblocks", "npcs", "items", "scenes"] {
            std::fs::create_dir_all(dir.path().join(d)).unwrap();
        }
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

    fn goblin() -> Combatant {
        Combatant {
            name: "Goblin".into(),
            kind: CombatantKind::Monster,
            hp: Some(7),
            ac: Some(15),
            initiative: Some(2),
            statblock: Some("statblocks/goblin.md".into()),
            tags: vec![],
            custom: Default::default(),
        }
    }

    #[test]
    fn lists_statblocks_and_encounters() {
        let dir = setup();
        std::fs::write(dir.path().join("statblocks/goblin.md"), GOBLIN).unwrap();
        std::fs::write(dir.path().join("npcs/grashnak.md"), GRASHNAK).unwrap();
        std::fs::write(
            dir.path().join("encounters/ambush.md"),
            "---\ntype: encounter\ntitle: Ambush\ncombatants:\n  - name: A\n  - name: B\n---\n",
        )
        .unwrap();

        let blocks = list_statblocks(dir.path()).unwrap();
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].name, "Goblin");
        assert_eq!(blocks[0].actions, 1);
        assert_eq!(blocks[0].power, Some(1));

        let encs = list_encounters(dir.path()).unwrap();
        assert_eq!(encs.len(), 1);
        assert_eq!(encs[0].combatants, 2);
    }

    const GRASHNAK: &str = "---\ntype: statblock\nname: Grashnak\nkind: npc\nhp: 45\n---\n";

    #[test]
    fn create_encounter_with_first_combatant() {
        let dir = setup();
        let rel = create_encounter(dir.path(), "Goblin Raid", Some(&goblin())).unwrap();
        assert_eq!(rel, "encounters/goblin-raid.md");
        let encs = list_encounters(dir.path()).unwrap();
        assert_eq!(encs.len(), 1);
        assert_eq!(encs[0].combatants, 1);

        let rel2 = create_encounter(dir.path(), "Goblin Raid", None).unwrap();
        assert_ne!(rel, rel2);
        assert!(rel2.starts_with("encounters/goblin-raid-"));
    }

    #[test]
    fn add_combatant_updates_blueprint_and_live_state() {
        let dir = setup();
        let rel = create_encounter(dir.path(), "Ambush", None).unwrap();

        // Simulate a running fight: start then mutate.
        let mut state = encounter::fresh_state(dir.path(), &rel).unwrap();
        state.round = 2;
        state.combatants.push(CombatantState {
            id: "goblin".into(),
            name: "Goblin".into(),
            kind: CombatantKind::Monster,
            initiative: Some(12),
            initiative_mod: Some(2),
            hp: 7,
            max_hp: Some(7),
            temp_hp: 0,
            ac: Some(15),
            statblock: None,
            conditions: vec![],
        });
        encounter::save_state_at(dir.path(), &state).unwrap();

        let mut wolf = goblin();
        wolf.name = "Winter Wolf".into();
        wolf.hp = Some(75);
        let summary = add_combatant_to_encounter(dir.path(), &rel, &wolf).unwrap();
        assert_eq!(summary.combatants, 1);

        // Blueprint has the combatant now.
        let encs = list_encounters(dir.path()).unwrap();
        assert_eq!(encs[0].combatants, 1);

        // Live state gained the combatant too.
        let resumed = encounter::start_encounter_at(dir.path(), &rel).unwrap();
        assert!(resumed.restored);
        assert_eq!(resumed.state.round, 2);
        assert_eq!(resumed.state.combatants.len(), 2);
        assert_eq!(resumed.state.combatants[1].name, "Winter Wolf");
        assert_eq!(resumed.state.combatants[1].id, "winter-wolf");
    }

    #[test]
    fn create_statblock_routes_by_kind() {
        let dir = setup();
        let rel = create_statblock(dir.path(), "Ancient Wyrm", StatblockKind::Monster, Some(300), Some(19), Some(10)).unwrap();
        assert_eq!(rel, "statblocks/ancient-wyrm.md");
        let rel2 = create_statblock(dir.path(), "Volo", StatblockKind::Npc, None, None, None).unwrap();
        assert_eq!(rel2, "npcs/volo.md");
        let blocks = list_statblocks(dir.path()).unwrap();
        assert_eq!(blocks.len(), 2);
    }

    #[test]
    fn duplicate_and_delete() {
        let dir = setup();
        std::fs::write(dir.path().join("statblocks/goblin.md"), GOBLIN).unwrap();
        let copy_rel = duplicate_file(dir.path(), "statblocks/goblin.md").unwrap();
        assert_eq!(copy_rel, "statblocks/goblin-copy.md");
        let blocks = list_statblocks(dir.path()).unwrap();
        assert_eq!(blocks.len(), 2);
        assert!(blocks.iter().any(|b| b.name == "Goblin (Copy)"));

        delete_file(dir.path(), &copy_rel).unwrap();
        assert!(list_statblocks(dir.path()).unwrap().len() == 1);
    }

    #[test]
    fn delete_encounter_drops_state() {
        let dir = setup();
        let rel = create_encounter(dir.path(), "Ambush", None).unwrap();
        let _ = encounter::fresh_state(dir.path(), &rel).unwrap();
        encounter::save_state_at(dir.path(), &encounter::fresh_state(dir.path(), &rel).unwrap()).unwrap();
        assert!(dir.path().join(".state/encounters/ambush.json").is_file());
        delete_file(dir.path(), &rel).unwrap();
        assert!(!dir.path().join(".state/encounters/ambush.json").exists());
    }

    #[test]
    fn rejects_traversal_and_empty_names() {
        let dir = setup();
        assert!(create_encounter(dir.path(), "  ", None).is_err());
        assert!(create_statblock(dir.path(), "", StatblockKind::Npc, None, None, None).is_err());
        let mut evil = goblin();
        assert!(add_combatant_to_encounter(dir.path(), "../outside.md", &evil).is_err());
        evil.name = "  ".into();
        assert!(add_combatant_to_encounter(dir.path(), "encounters/x.md", &evil).is_err());
    }
}
