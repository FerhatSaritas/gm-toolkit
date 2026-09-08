//! Generator engine: data-driven table packs.
//!
//! Packs are YAML files describing named tables (plain or weighted entries),
//! optional structured `fields` and one or more `templates` whose `{key}`
//! placeholders resolve to rendered fields first, then weighted-random table
//! picks. Built-in packs ship embedded; users may add packs under
//! `<campaign>/generators/*.yaml`. A broken pack never crashes the app — it
//! is listed with an error and simply unusable.

use crate::catalog::{self, StatblockSummary};
use crate::doc::{Combatant, EncounterMeta, ItemMeta, StatblockMeta};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

// ---------------------------------------------------------------------------
// Pack model
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PackKind {
    Npc,
    Item,
    Encounter,
}

impl PackKind {
    pub fn parse(s: &str) -> Result<PackKind, String> {
        match s.to_lowercase().as_str() {
            "npc" => Ok(PackKind::Npc),
            "item" => Ok(PackKind::Item),
            "encounter" => Ok(PackKind::Encounter),
            other => Err(format!("Unknown pack kind “{other}” (npc | item | encounter)")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WeightedEntry {
    pub weight: u32,
    pub value: String,
}

#[derive(Debug, Clone, Default)]
pub struct Pack {
    pub name: String,
    pub kind: Option<PackKind>,
    pub tables: BTreeMap<String, Vec<WeightedEntry>>,
    pub templates: BTreeMap<String, String>,
    pub fields: BTreeMap<String, String>,
}

// Untagged: plain strings or {weight, value} maps.
#[derive(Deserialize)]
#[serde(untagged)]
enum EntryRepr {
    Plain(String),
    Weighted {
        #[serde(default = "one")]
        weight: u32,
        value: String,
    },
}

fn one() -> u32 {
    1
}

impl<'de> Deserialize<'de> for WeightedEntry {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        match EntryRepr::deserialize(d)? {
            EntryRepr::Plain(value) => Ok(WeightedEntry { weight: 1, value }),
            EntryRepr::Weighted { weight, value } => Ok(WeightedEntry { weight, value }),
        }
    }
}

#[derive(Deserialize)]
struct PackRepr {
    name: Option<String>,
    kind: Option<String>,
    #[serde(default)]
    tables: BTreeMap<String, Vec<WeightedEntry>>,
    #[serde(default)]
    templates: BTreeMap<String, String>,
    #[serde(default)]
    fields: BTreeMap<String, String>,
}

impl Pack {
    pub fn parse(yaml: &str) -> Result<Pack, String> {
        let repr: PackRepr = serde_yaml_ng::from_str(yaml).map_err(|e| format!("Invalid pack YAML: {e}"))?;
        let kind = match &repr.kind {
            Some(k) => Some(PackKind::parse(k)?),
            None => None,
        };
        if repr.tables.is_empty() {
            return Err("Pack has no tables".to_string());
        }
        if repr.templates.is_empty() {
            return Err("Pack has no templates".to_string());
        }
        for (tname, entries) in &repr.tables {
            if entries.is_empty() {
                return Err(format!("Table “{tname}” is empty"));
            }
            if entries.iter().all(|e| e.weight == 0) {
                return Err(format!("Table “{tname}” has zero total weight"));
            }
        }
        Ok(Pack {
            name: repr.name.unwrap_or_else(|| "Untitled Pack".to_string()),
            kind,
            tables: repr.tables,
            templates: repr.templates,
            fields: repr.fields,
        })
    }
}

// ---------------------------------------------------------------------------
// Listing
// ---------------------------------------------------------------------------

pub const BUILTIN_PACKS: &[(&str, &str)] = &[
    ("npc", include_str!("../packs/npc.yaml")),
    ("item", include_str!("../packs/item.yaml")),
    ("encounter", include_str!("../packs/encounter.yaml")),
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackInfo {
    pub id: String,
    pub source: String, // "builtin" | "user"
    pub name: String,
    pub kind: Option<PackKind>,
    pub templates: Vec<String>,
    pub tables: Vec<String>,
    pub error: Option<String>,
}

pub fn list_packs(campaign_root: &Path) -> Vec<PackInfo> {
    let mut out = Vec::new();
    for (key, yaml) in BUILTIN_PACKS {
        match Pack::parse(yaml) {
            Ok(p) => out.push(PackInfo {
                id: format!("builtin:{key}"),
                source: "builtin".to_string(),
                name: p.name,
                kind: p.kind,
                templates: p.templates.keys().cloned().collect(),
                tables: p.tables.keys().cloned().collect(),
                error: None,
            }),
            Err(e) => out.push(PackInfo {
                id: format!("builtin:{key}"),
                source: "builtin".to_string(),
                name: format!("Built-in {key}"),
                kind: None,
                templates: Vec::new(),
                tables: Vec::new(),
                error: Some(e),
            }),
        }
    }

    let dir = campaign_root.join("generators");
    if let Ok(entries) = std::fs::read_dir(&dir) {
        let mut files: Vec<_> = entries
            .flatten()
            .filter(|e| {
                e.path()
                    .extension()
                    .map(|x| x == "yaml" || x == "yml")
                    .unwrap_or(false)
            })
            .map(|e| e.path())
            .collect();
        files.sort();
        for path in files {
            let id = format!("user:{}", path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default());
            match std::fs::read_to_string(&path)
                .map_err(|e| e.to_string())
                .and_then(|raw| Pack::parse(&raw))
            {
                Ok(p) => out.push(PackInfo {
                    id: id.clone(),
                    source: "user".to_string(),
                    name: p.name,
                    kind: p.kind,
                    templates: p.templates.keys().cloned().collect(),
                    tables: p.tables.keys().cloned().collect(),
                    error: None,
                }),
                Err(e) => out.push(PackInfo {
                    id,
                    source: "user".to_string(),
                    name: path
                        .file_stem()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_else(|| "User pack".to_string()),
                    kind: None,
                    templates: Vec::new(),
                    tables: Vec::new(),
                    error: Some(e),
                }),
            }
        }
    }
    out
}

fn load_pack(campaign_root: &Path, pack_id: &str) -> Result<Pack, String> {
    let (source, key) = pack_id
        .split_once(':')
        .ok_or_else(|| format!("Malformed pack id “{pack_id}”"))?;
    match source {
        "builtin" => {
            let yaml = BUILTIN_PACKS
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, y)| *y)
                .ok_or_else(|| format!("No built-in pack “{key}”"))?;
            Pack::parse(yaml)
        }
        "user" => {
            // File name is validated: no separators/traversal.
            if key.is_empty() || key.contains('/') || key.contains('\\') || key.contains("..") {
                return Err("Malformed pack file name".to_string());
            }
            let raw = std::fs::read_to_string(campaign_root.join("generators").join(key))
                .map_err(|e| format!("Cannot read pack: {e}"))?;
            Pack::parse(&raw)
        }
        other => Err(format!("Unknown pack source “{other}”")),
    }
}

// ---------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Generated {
    pub pack_id: String,
    pub pack_name: String,
    pub kind: Option<PackKind>,
    pub template: String,
    pub text: String,
    pub fields: BTreeMap<String, String>,
}

fn pick<'a>(entries: &'a [WeightedEntry], rng: &mut impl rand::Rng) -> &'a str {
    let total: u32 = entries.iter().map(|e| e.weight).sum();
    let mut roll = rng.random_range(0..total.max(1));
    for e in entries {
        if roll < e.weight {
            return &e.value;
        }
        roll -= e.weight;
    }
    entries
        .last()
        .map(|e| e.value.as_str())
        .unwrap_or_default()
}

fn render(
    template: &str,
    pack: &Pack,
    fields: Option<&BTreeMap<String, String>>,
    rng: &mut impl rand::Rng,
) -> Result<String, String> {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        let Some(close) = after.find('}') else {
            return Err("Template has an unclosed “{” placeholder".to_string());
        };
        let key = &after[..close];
        if let Some(v) = fields.and_then(|f| f.get(key)) {
            out.push_str(v);
        } else if let Some(entries) = pack.tables.get(key) {
            out.push_str(pick(entries, rng));
        } else {
            return Err(format!("Unknown placeholder “{{{key}}}”"));
        }
        rest = &after[close + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

pub fn generate(
    campaign_root: &Path,
    pack_id: &str,
    template_name: Option<&str>,
) -> Result<Generated, String> {
    let pack = load_pack(campaign_root, pack_id)?;
    let mut rng = rand::rng();

    // Fields render first (tables only).
    let mut fields = BTreeMap::new();
    for (k, tpl) in &pack.fields {
        fields.insert(k.clone(), render(tpl, &pack, None, &mut rng)?);
    }

    let tname = match template_name {
        Some(t) if pack.templates.contains_key(t) => t.to_string(),
        Some(t) => return Err(format!("Pack has no template “{t}”")),
        None => pack
            .templates
            .keys()
            .next()
            .cloned()
            .ok_or_else(|| "Pack has no templates".to_string())?,
    };
    let text = render(
        &pack.templates[&tname],
        &pack,
        Some(&fields),
        &mut rng,
    )?;

    Ok(Generated {
        pack_id: pack_id.to_string(),
        pack_name: pack.name,
        kind: pack.kind,
        template: tname,
        text,
        fields,
    })
}

// ---------------------------------------------------------------------------
// Encounter drafting from the bestiary
// ---------------------------------------------------------------------------

pub fn generate_encounter_draft(
    campaign_root: &Path,
    party_level: u32,
    band: u32,
    count: usize,
) -> Result<Vec<StatblockSummary>, String> {
    if count == 0 || count > 50 {
        return Err("Creature count must be 1–50".to_string());
    }
    let all = catalog::list_statblocks(campaign_root)?;
    let pool: Vec<&StatblockSummary> = all
        .iter()
        .filter(|s| {
            s.power
                .map(|p| (p as i32 - party_level as i32).abs() <= band as i32)
                .unwrap_or(false)
        })
        .collect();
    if pool.is_empty() {
        return Err(format!(
            "No creatures with power {lo}–{hi}. Widen the band or add creatures.",
            lo = party_level.saturating_sub(band),
            hi = party_level + band
        ));
    }

    let mut rng = rand::rng();
    let mut picks: Vec<StatblockSummary> = Vec::with_capacity(count);
    let mut last_idx: Option<usize> = None;
    for _ in 0..count {
        let idx = rng.random_range(0..pool.len());
        // Avoid immediate repeats when the pool allows it.
        let idx = if pool.len() > 1 && last_idx == Some(idx) {
            (idx + 1) % pool.len()
        } else {
            idx
        };
        picks.push(pool[idx].clone());
        last_idx = Some(idx);
    }
    Ok(picks)
}

pub fn save_encounter_draft(
    campaign_root: &Path,
    title: &str,
    party_level: Option<u32>,
    combatants: &[Combatant],
    body: &str,
) -> Result<String, String> {
    if title.trim().is_empty() {
        return Err("Encounter needs a title".to_string());
    }
    let meta = EncounterMeta {
        doc_type: "encounter".into(),
        title: title.trim().to_string(),
        party_level,
        combatants: combatants.to_vec(),
        tags: vec![],
        custom: Default::default(),
    };
    catalog::write_encounter(campaign_root, &meta, body)
}

pub fn save_generated_statblock(
    campaign_root: &Path,
    meta: &StatblockMeta,
    body: &str,
) -> Result<String, String> {
    catalog::write_statblock(campaign_root, meta, body)
}

pub fn save_generated_item(
    campaign_root: &Path,
    meta: &ItemMeta,
    body: &str,
) -> Result<String, String> {
    if meta.name.trim().is_empty() {
        return Err("Item needs a name".to_string());
    }
    catalog::write_item(campaign_root, meta, body)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::campaign::slugify;
    use crate::doc::{CombatantKind, StatblockKind};

    fn setup() -> tempdir::TempDir {
        let dir = tempdir::TempDir::new("gmgen").unwrap();
        for d in ["encounters", "statblocks", "npcs", "items", "party"] {
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

    #[test]
    fn parses_weighted_and_plain_entries() {
        let yaml = "name: T\nkind: npc\ntables:\n  a: [x, y]\n  b:\n    - weight: 3\n      value: heavy\n    - light\ntemplates:\n  default: \"{a} {b}\"\n";
        let pack = Pack::parse(yaml).unwrap();
        assert_eq!(pack.tables["a"].len(), 2);
        assert_eq!(pack.tables["b"][0].weight, 3);
        assert_eq!(pack.tables["b"][1].weight, 1);
        assert_eq!(pack.kind, Some(PackKind::Npc));
    }

    #[test]
    fn rejects_broken_packs() {
        assert!(Pack::parse("name: T\ntables: {}\ntemplates:\n  d: x\n").is_err());
        assert!(Pack::parse("name: T\ntables:\n  a: [x]\ntemplates: {}\n").is_err());
        assert!(Pack::parse("name: T\ntables:\n  a: []\ntemplates:\n  d: x\n").is_err());
        assert!(Pack::parse("name: T\nkind: spell\ntables:\n  a: [x]\ntemplates:\n  d: x\n").is_err());
    }

    #[test]
    fn builtin_packs_generate() {
        for (key, yaml) in BUILTIN_PACKS {
            let pack = Pack::parse(yaml).unwrap_or_else(|e| panic!("{key}: {e}"));
            assert!(pack.kind.is_some(), "{key} missing kind");
        }
        let dir = setup();
        for key in ["builtin:npc", "builtin:item", "builtin:encounter"] {
            for _ in 0..25 {
                let g = generate(dir.path(), key, None).unwrap();
                assert!(!g.text.contains('{'), "{key} left a placeholder: {}", g.text);
            }
        }
    }

    #[test]
    fn item_fields_are_rendered() {
        let dir = setup();
        let g = generate(dir.path(), "builtin:item", None).unwrap();
        assert!(g.fields.contains_key("name"));
        assert!(!g.fields["name"].is_empty());
        assert!(["common", "uncommon", "rare", "very rare", "legendary"].contains(&g.fields["rarity"].as_str()));
    }

    #[test]
    fn unknown_placeholder_is_an_error() {
        let dir = setup();
        let raw = "name: Bad\nkind: npc\ntables:\n  a: [x]\ntemplates:\n  default: \"{nope}\"\n";
        std::fs::create_dir_all(dir.path().join("generators")).unwrap();
        std::fs::write(dir.path().join("generators/bad.yaml"), raw).unwrap();
        let err = generate(dir.path(), "user:bad.yaml", None).unwrap_err();
        assert!(err.contains("nope"));
    }

    #[test]
    fn user_pack_listing_reports_errors() {
        let dir = setup();
        std::fs::create_dir_all(dir.path().join("generators")).unwrap();
        std::fs::write(dir.path().join("generators/good.yaml"), "name: Good\nkind: npc\ntables:\n  a: [x]\ntemplates:\n  default: \"{a}\"\n").unwrap();
        std::fs::write(dir.path().join("generators/broken.yaml"), ":::not yaml [").unwrap();
        let packs = list_packs(dir.path());
        assert_eq!(packs.len(), 5); // 3 builtin + 2 user
        let broken = packs.iter().find(|p| p.name == "broken").unwrap();
        assert!(broken.error.is_some());
        let good = packs.iter().find(|p| p.name == "Good").unwrap();
        assert!(good.error.is_none());
    }

    #[test]
    fn encounter_draft_respects_power_band() {
        let dir = setup();
        for (name, power) in [("Weakling", 1), ("Goblin", 2), ("Orc", 3), ("Troll", 4), ("Dragon", 9)] {
            let meta = StatblockMeta {
                doc_type: "statblock".into(),
                name: name.into(),
                kind: StatblockKind::Monster,
                power: Some(power),
                ..Default::default()
            };
            catalog::write_statblock(dir.path(), &meta, "").unwrap();
        }
        let picks = generate_encounter_draft(dir.path(), 3, 1, 3).unwrap();
        assert_eq!(picks.len(), 3);
        assert!(picks.iter().all(|p| p.power.unwrap() >= 2 && p.power.unwrap() <= 4));

        let picks = generate_encounter_draft(dir.path(), 3, 0, 2).unwrap();
        assert!(picks.iter().all(|p| p.power == Some(3)));

        assert!(generate_encounter_draft(dir.path(), 6, 0, 1).is_err()); // nothing at power 6
        assert!(generate_encounter_draft(dir.path(), 3, 1, 0).is_err());
    }

    #[test]
    fn save_paths_are_unique_and_valid() {
        let dir = setup();
        let npc_meta = StatblockMeta {
            doc_type: "statblock".into(),
            name: "Corvin".into(),
            kind: StatblockKind::Npc,
            ..Default::default()
        };
        let r1 = save_generated_statblock(dir.path(), &npc_meta, "text").unwrap();
        let r2 = save_generated_statblock(dir.path(), &npc_meta, "text").unwrap();
        assert_eq!(r1, "npcs/corvin.md");
        assert_eq!(r2, "npcs/corvin-2.md");

        let item_meta = ItemMeta {
            doc_type: "item".into(),
            name: "Gilded dagger".into(),
            rarity: Some("rare".into()),
            ..Default::default()
        };
        let rel = save_generated_item(dir.path(), &item_meta, "**Effect:** shiny").unwrap();
        assert_eq!(rel, "items/gilded-dagger.md");

        let combatant = Combatant {
            name: "Goblin".into(),
            kind: CombatantKind::Monster,
            hp: Some(7),
            ..Default::default()
        };
        let rel = save_encounter_draft(dir.path(), "Goblin Raid", Some(3), &[combatant], "# Setup\n").unwrap();
        assert_eq!(rel, format!("encounters/{}.md", slugify("Goblin Raid")));
        assert!(dir.path().join(&rel).is_file());
    }
}
