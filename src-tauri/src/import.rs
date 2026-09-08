//! Statblock import: markdown passthrough + lenient JSON mapping.
//!
//! Accepted sources:
//! - `.md` files already matching the blueprint schema (frontmatter required).
//! - `.json` files from common statblock formats. Field names are matched
//!   leniently (`hp` / `average_hp` / `hit_points`, `ac` / `armor_class`,
//!   abilities as map or 6-number array, `desc`/`entries` for action text…).
//!   Unknown top-level keys are preserved in `custom` so nothing is lost.
//!
//! Nothing is written until the caller commits a previewed statblock.

use crate::catalog::write_statblock;
use crate::campaign::slugify;
use crate::doc::{NamedText, StatblockKind, StatblockMeta};
use serde::Serialize;
use serde_json::Value as J;
use std::collections::BTreeMap;
use std::path::Path;

use serde_yaml_ng::Value as Y;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportPreview {
    pub source_path: String,
    pub format: String,
    pub statblock: StatblockMeta,
    pub body: String,
    pub warnings: Vec<String>,
    pub suggested_path: String,
}

pub fn preview(source_path: &str) -> Result<ImportPreview, String> {
    let raw = std::fs::read_to_string(source_path)
        .map_err(|e| format!("Cannot read import file: {e}"))?;
    let ext = Path::new(source_path)
        .extension()
        .map(|x| x.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let (mut statblock, body, mut warnings, format) = match ext.as_str() {
        "json" => {
            let json: J = serde_json::from_str(&raw)
                .map_err(|e| format!("Invalid JSON: {e}"))?;
            map_json(&json)?
        }
        "md" | "markdown" => {
            let doc = crate::doc::parse_doc(source_path, &raw)?;
            if doc.kind != crate::doc::DocKind::Statblock {
                return Err("Markdown file is not a statblock blueprint".to_string());
            }
            (
                doc.statblock.clone().unwrap_or_default(),
                doc.body,
                Vec::new(),
                "md",
            )
        }
        other => return Err(format!("Unsupported import format “.{other}” (use .md or .json)")),
    };

    if statblock.name.trim().is_empty() {
        statblock.name = fallback_name(source_path);
        warnings.push("Name missing — derived from filename".to_string());
    }
    if statblock.hp.is_none() {
        warnings.push("No HP found".to_string());
    }
    if statblock.ac.is_none() {
        warnings.push("No AC found".to_string());
    }

    let suggested_path = format!(
        "{}/{}.md",
        match statblock.kind {
            StatblockKind::Npc => "npcs",
            StatblockKind::Player => "party",
            StatblockKind::Monster => "statblocks",
        },
        slugify(&statblock.name)
    );

    Ok(ImportPreview {
        source_path: source_path.to_string(),
        format: format.to_string(),
        statblock,
        body,
        warnings,
        suggested_path,
    })
}

fn fallback_name(path: &str) -> String {
    Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| "Imported Creature".to_string())
}

pub fn commit(
    campaign_root: &Path,
    statblock: &StatblockMeta,
    body: &str,
) -> Result<String, String> {
    if statblock.name.trim().is_empty() {
        return Err("Imported statblock needs a name".to_string());
    }
    write_statblock(campaign_root, statblock, body)
}

// ---------------------------------------------------------------------------
// JSON mapping
// ---------------------------------------------------------------------------

type Mapped = (StatblockMeta, String, Vec<String>, &'static str);

fn map_json(json: &J) -> Result<Mapped, String> {
    let obj = json
        .as_object()
        .ok_or_else(|| "JSON root must be an object".to_string())?;

    let mut meta = StatblockMeta {
        doc_type: "statblock".into(),
        ..Default::default()
    };
    let mut warnings: Vec<String> = Vec::new();
    let mut custom: BTreeMap<String, Y> = BTreeMap::new();

    // Name
    if let Some(v) = first_str(obj, &["name", "title", "creature"]) {
        meta.name = v;
    }

    // Kind — accept explicit kinds; map creatureType/size hints.
    for key in ["kind", "type", "creature_type", "creatureType", "category"] {
        if let Some(v) = obj.get(key).and_then(J::as_str) {
            let lower = v.to_lowercase();
            if lower.contains("npc") || lower.contains("character") {
                meta.kind = StatblockKind::Npc;
            } else if lower.contains("player") {
                meta.kind = StatblockKind::Player;
            }
            break;
        }
    }

    // HP: number, or object with average/mean.
    for key in ["hp", "average_hp", "hitPoints", "hit_points", "hitPointAverage"] {
        if let Some(v) = obj.get(key) {
            if let Some(n) = as_num(v) {
                meta.hp = Some(n as i64);
            } else if let Some(o) = v.as_object() {
                meta.hp = o
                    .get("average")
                    .or_else(|| o.get("mean"))
                    .and_then(as_num)
                    .map(|n| n as i64);
            }
            break;
        }
    }

    // AC: number, or array of objects with value/ac.
    for key in ["ac", "armor_class", "armorClass"] {
        if let Some(v) = obj.get(key) {
            meta.ac = ac_from(v);
            break;
        }
    }

    // Abilities: map or 6-number array (parsed first — initiative may derive from DEX).
    if let Some(v) = first_of(obj, &["abilities", "ability_scores", "abilityScores", "stats"]) {
        meta.abilities = abilities_from(v);
        if meta.abilities.is_none() {
            warnings.push("Could not read ability scores".to_string());
        }
    }

    // Initiative: number, else derive from dex.
    if let Some(n) = ["initiative", "init"]
        .iter()
        .find_map(|k| obj.get(*k).and_then(as_num))
    {
        meta.initiative = Some(n as i64);
    } else if let Some(dex) = meta.abilities.as_ref().and_then(|a| a.dexterity) {
        meta.initiative = Some((dex - 10).div_euclid(2));
        warnings.push("Initiative derived from DEX".to_string());
    }

    // Speed: string, number, or object of movement types.
    if let Some(v) = first_of(obj, &["speed", "movement"]) {
        meta.speed = speed_from(v);
    }

    // Power rating: our 1–10 scale; CR is heuristically mapped.
    if let Some(n) = ["power", "rating"]
        .iter()
        .find_map(|k| obj.get(*k).and_then(as_num))
    {
        meta.power = clamp_power(n);
    } else if let Some(n) = ["cr", "challenge_rating", "challengeRating", "level"]
        .iter()
        .find_map(|k| obj.get(*k).and_then(as_num))
    {
        meta.power = clamp_power(n);
        warnings.push(format!("Mapped CR/level {n} → power rating {}", meta.power.unwrap_or(1)));
    }

    // Trait-like sections.
    if let Some(list) = entry_list(obj.get("traits").or_else(|| obj.get("special_abilities"))) {
        meta.traits = list;
    }
    if let Some(list) = entry_list(obj.get("actions")) {
        meta.actions = list;
    }
    if let Some(list) = entry_list(obj.get("reactions")) {
        meta.reactions = list;
    }
    if let Some(list) = entry_list(obj.get("bonus_actions")) {
        meta.traits.extend(list);
        warnings.push("Bonus actions merged into traits".to_string());
    }
    if let Some(list) = entry_list(obj.get("legendary_actions").or_else(|| obj.get("legendaryActions"))) {
        meta.actions.extend(list);
        warnings.push("Legendary actions merged into actions".to_string());
    }

    // Tags
    if let Some(v) = obj.get("tags") {
        if let Some(arr) = v.as_array() {
            meta.tags = arr
                .iter()
                .filter_map(|x| x.as_str().map(str::to_string))
                .collect();
        }
    }

    // Unknown top-level keys → custom (preserved on round-trip).
    let known: [&str; 24] = [
        "name", "title", "creature", "kind", "type", "creature_type", "creatureType", "category",
        "hp", "average_hp", "hitPoints", "hit_points", "hitPointAverage", "ac", "armor_class",
        "armorClass", "initiative", "init", "speed", "movement", "power", "rating", "cr",
        "challenge_rating",
    ];
    let known_extra = [
        "challengeRating",
        "level",
        "abilities",
        "ability_scores",
        "abilityScores",
        "stats",
        "traits",
        "special_abilities",
        "actions",
        "reactions",
        "bonus_actions",
        "legendary_actions",
        "legendaryActions",
        "tags",
    ];
    let known: [&str; 38] = [
        known.as_slice(),
        known_extra.as_slice(),
    ]
    .concat()
    .try_into()
    .unwrap();

    let mut unknown = 0usize;
    for (k, v) in obj {
        if !known.contains(&k.as_str()) {
            if let Ok(y) = serde_json::from_value::<Y>(v.clone()) {
                custom.insert(k.clone(), y);
                unknown += 1;
            }
        }
    }
    if unknown > 0 {
        warnings.push(format!("{unknown} unknown field(s) preserved as custom"));
    }
    meta.custom = custom;

    Ok((meta, String::new(), warnings, "json"))
}

fn first_str<'a>(obj: &'a serde_json::Map<String, J>, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|k| obj.get(*k).and_then(J::as_str).map(str::to_string))
}

fn first_of<'a>(obj: &'a serde_json::Map<String, J>, keys: &[&str]) -> Option<&'a J> {
    keys.iter().find_map(|k| obj.get(*k))
}

fn as_num(v: &J) -> Option<f64> {
    v.as_f64()
}

fn clamp_power(n: f64) -> Option<u8> {
    Some(n.round().clamp(1.0, 10.0) as u8)
}

fn ac_from(v: &J) -> Option<i64> {
    if let Some(n) = v.as_i64() {
        return Some(n);
    }
    if let Some(n) = v.as_f64() {
        return Some(n as i64);
    }
    if let Some(arr) = v.as_array() {
        for item in arr {
            if let Some(n) = item.as_i64().or_else(|| item.as_f64().map(|f| f as i64)) {
                return Some(n);
            }
            if let Some(o) = item.as_object() {
                if let Some(n) = o
                    .get("value")
                    .or_else(|| o.get("ac"))
                    .and_then(J::as_i64)
                {
                    return Some(n);
                }
            }
        }
    }
    None
}

fn speed_from(v: &J) -> Option<String> {
    if let Some(s) = v.as_str() {
        return Some(s.trim().to_string());
    }
    if let Some(n) = v.as_f64() {
        return Some(format!("{n} ft"));
    }
    if let Some(o) = v.as_object() {
        let mut parts = Vec::new();
        for (kind, val) in o {
            if let Some(s) = val.as_str() {
                parts.push(format!("{kind} {}", s.trim()));
            } else if let Some(n) = val.as_f64() {
                parts.push(format!("{kind} {n} ft"));
            }
        }
        if !parts.is_empty() {
            return Some(parts.join(", "));
        }
    }
    None
}

const ABILITY_KEYS: [(&str, &str); 12] = [
    ("str", "strength"),
    ("strength", "strength"),
    ("dex", "dexterity"),
    ("dexterity", "dexterity"),
    ("con", "constitution"),
    ("constitution", "constitution"),
    ("int", "intelligence"),
    ("int_", "intelligence"),
    ("intelligence", "intelligence"),
    ("wis", "wisdom"),
    ("wisdom", "wisdom"),
    ("cha", "charisma"),
];

fn ability_key(k: &str) -> Option<&'static str> {
    let lower = k.to_lowercase();
    ABILITY_KEYS
        .iter()
        .find(|(alias, _)| *alias == lower)
        .map(|(_, canonical)| *canonical)
}

fn abilities_from(v: &J) -> Option<crate::doc::Abilities> {
    use crate::doc::Abilities;
    let mut ab = Abilities::default();
    if let Some(o) = v.as_object() {
        for (k, val) in o {
            let Some(key) = ability_key(k) else {
                continue;
            };
            let n = match val {
                J::Number(_) => as_num(val),
                J::Object(_) => val
                    .as_object()
                    .and_then(|o| {
                        o.get("score")
                            .or_else(|| o.get("value"))
                            .or_else(|| o.get("mod"))
                    })
                    .and_then(as_num),
                _ => None,
            };
            if let Some(n) = n {
                set_ability(&mut ab, key, n as i64);
            }
        }
        return Some(ab);
    }
    if let Some(arr) = v.as_array() {
        if arr.len() == 6 {
            let nums: Option<Vec<i64>> = arr
                .iter()
                .map(|x| x.as_f64().map(|n| n as i64))
                .collect();
            if let Some(nums) = nums {
                ab.strength = Some(nums[0]);
                ab.dexterity = Some(nums[1]);
                ab.constitution = Some(nums[2]);
                ab.intelligence = Some(nums[3]);
                ab.wisdom = Some(nums[4]);
                ab.charisma = Some(nums[5]);
                return Some(ab);
            }
        }
    }
    None
}

fn set_ability(ab: &mut crate::doc::Abilities, key: &str, value: i64) {
    match key {
        "strength" => ab.strength = Some(value),
        "dexterity" => ab.dexterity = Some(value),
        "constitution" => ab.constitution = Some(value),
        "intelligence" => ab.intelligence = Some(value),
        "wisdom" => ab.wisdom = Some(value),
        "charisma" => ab.charisma = Some(value),
        _ => {}
    }
}

fn entry_list(v: Option<&J>) -> Option<Vec<NamedText>> {
    let arr = v?.as_array()?;
    if arr.is_empty() {
        return None;
    }
    let mut out = Vec::new();
    for item in arr {
        let Some(o) = item.as_object() else { continue };
        let name = o
            .get("name")
            .and_then(J::as_str)
            .unwrap_or("Unnamed")
            .to_string();
        let text = entry_text(o.get("desc").or_else(|| o.get("entries")).or_else(|| o.get("text")));
        out.push(NamedText { name, text });
    }
    Some(out)
}

fn entry_text(v: Option<&J>) -> String {
    match v {
        Some(J::String(s)) => s.clone(),
        Some(J::Array(arr)) => arr
            .iter()
            .filter_map(|x| x.as_str())
            .collect::<Vec<_>>()
            .join(" "),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn campaign() -> tempdir::TempDir {
        let dir = tempdir::TempDir::new("gmimp").unwrap();
        for d in ["statblocks", "npcs", "party", "encounters"] {
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

    const OPEN5E: &str = r#"{
        "name": "Goblin",
        "size": "Small",
        "type": "goblinoid",
        "hit_points": { "average": 7, "formula": "2d6" },
        "armor_class": 15,
        "speed": { "walk": 30 },
        "abilities": { "str": 8, "dex": 14, "con": 10, "int": 10, "wis": 8, "cha": 8 },
        "special_abilities": [
            { "name": "Nimble Escape", "desc": "Can Disengage or Hide as a bonus action." }
        ],
        "actions": [
            { "name": "Scimitar", "desc": "Melee: +4 to hit, 1d6+2 slashing." }
        ],
        "cr": 0.25,
        "senses": { "darkvision": 60 }
    }"#;

    #[test]
    fn maps_open5e_style_json() {
        let json: J = serde_json::from_str(OPEN5E).unwrap();
        let (meta, _body, warnings, format) = map_json(&json).unwrap();
        assert_eq!(format, "json");
        assert_eq!(meta.name, "Goblin");
        assert_eq!(meta.hp, Some(7));
        assert_eq!(meta.ac, Some(15));
        assert_eq!(meta.speed.as_deref(), Some("walk 30 ft"));
        let ab = meta.abilities.unwrap();
        assert_eq!(ab.strength, Some(8));
        assert_eq!(ab.dexterity, Some(14));
        assert_eq!(meta.traits.len(), 1);
        assert_eq!(meta.traits[0].name, "Nimble Escape");
        assert_eq!(meta.actions.len(), 1);
        assert_eq!(meta.actions[0].text, "Melee: +4 to hit, 1d6+2 slashing.");
        assert_eq!(meta.power, Some(1)); // cr 0.25 → 1
        assert!(warnings.iter().any(|w| w.contains("CR/level")));
        assert!(warnings.iter().any(|w| w.contains("unknown field")));
        assert!(meta.custom.contains_key("senses"));
    }

    #[test]
    fn maps_array_abilities_and_keeps() {
        let json: J = serde_json::from_str(
            r#"{ "name": "Brute", "armor_class": [{"value": 14}], "hitPoints": 40,
                 "abilities": [18, 14, 16, 8, 10, 12],
                 "actions": [{"name": "Smash", "entries": ["Big hit.", "1d12+4"]}],
                 "legendary_actions": [{"name": "Roar", "desc": "Scary."}] }"#,
        )
        .unwrap();
        let (meta, _b, warnings, _) = map_json(&json).unwrap();
        assert_eq!(meta.ac, Some(14));
        assert_eq!(meta.hp, Some(40));
        let ab = meta.abilities.unwrap();
        assert_eq!((ab.strength, ab.dexterity, ab.constitution), (Some(18), Some(14), Some(16)));
        assert_eq!((ab.intelligence, ab.wisdom, ab.charisma), (Some(8), Some(10), Some(12)));
        assert_eq!(meta.actions[0].name, "Smash");
        assert_eq!(meta.actions[0].text, "Big hit. 1d12+4");
        assert_eq!(meta.actions.len(), 2); // legendary merged
        assert!(warnings.iter().any(|w| w.contains("Legendary")));
    }

    #[test]
    fn derives_initiative_from_dex() {
        let json: J = serde_json::from_str(r#"{ "name": "X", "abilities": { "dex": 16 } }"#).unwrap();
        let (meta, _b, warnings, _) = map_json(&json).unwrap();
        assert_eq!(meta.initiative, Some(3));
        assert!(warnings.iter().any(|w| w.contains("derived from DEX")));
    }

    #[test]
    fn name_falls_back_and_missing_stats_warn() {
        let dir = campaign();
        let p = dir.path().join("mystery-thing.json");
        std::fs::write(&p, r#"{ "hp": 12 }"#).unwrap();
        let preview = preview(p.to_str().unwrap()).unwrap();
        assert_eq!(preview.statblock.name, "mystery-thing");
        assert!(preview.warnings.iter().any(|w| w.contains("Name missing")));
        assert!(preview.warnings.iter().any(|w| w.contains("No AC")));
        assert_eq!(preview.suggested_path, "statblocks/mystery-thing.md");
    }

    #[test]
    fn markdown_passthrough() {
        let dir = campaign();
        let p = dir.path().join("goblin.md");
        std::fs::write(
            &p,
            "---\ntype: statblock\nname: Goblin\nkind: npc\nhp: 7\n---\nNotes.\n",
        )
        .unwrap();
        let preview = preview(p.to_str().unwrap()).unwrap();
        assert_eq!(preview.format, "md");
        assert_eq!(preview.statblock.kind, StatblockKind::Npc);
        assert_eq!(preview.suggested_path, "npcs/goblin.md");
        assert_eq!(preview.body, "Notes.\n");
    }

    #[test]
    fn rejects_bad_sources() {
        let dir = campaign();
        let p = dir.path().join("x.txt");
        std::fs::write(&p, "hello").unwrap();
        assert!(preview(p.to_str().unwrap()).is_err());

        let p = dir.path().join("bad.json");
        std::fs::write(&p, "{nope").unwrap();
        assert!(preview(p.to_str().unwrap()).is_err());

        let p = dir.path().join("arr.json");
        std::fs::write(&p, "[1,2]").unwrap();
        assert!(preview(p.to_str().unwrap()).is_err());

        let p = dir.path().join("scene.md");
        std::fs::write(&p, "---\ntype: scene\ntitle: X\n---\n").unwrap();
        assert!(preview(p.to_str().unwrap()).is_err());
    }

    #[test]
    fn commit_writes_file_with_unique_name() {
        let dir = campaign();
        let json: J = serde_json::from_str(OPEN5E).unwrap();
        let (meta, body, _, _) = map_json(&json).unwrap();
        let rel1 = commit(dir.path(), &meta, &body).unwrap();
        assert_eq!(rel1, "statblocks/goblin.md");
        let rel2 = commit(dir.path(), &meta, &body).unwrap();
        assert_eq!(rel2, "statblocks/goblin-2.md");
        // Round-trip: written file parses back with the same fields.
        let raw = std::fs::read_to_string(dir.path().join(&rel1)).unwrap();
        let doc = crate::doc::parse_doc(&rel1, &raw).unwrap();
        assert_eq!(doc.statblock.unwrap().hp, Some(7));
    }
}
