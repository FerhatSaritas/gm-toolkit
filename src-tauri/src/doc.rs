//! Blueprint document model: Markdown + YAML frontmatter.
//!
//! Every campaign file carries a typed frontmatter block (`---` fenced YAML)
//! with a `type` tag: `scene` | `encounter` | `statblock` | `item`.
//! Unknown frontmatter keys are preserved via a flattened `custom` map so
//! serialization never destroys user data. Trailing whitespace is normalized
//! to a single trailing newline on save.

use serde::Deserialize;
use serde::Serialize;
use serde_yaml_ng::Value;
use std::collections::BTreeMap;

pub const DOC_TYPES: [&str; 4] = ["scene", "encounter", "statblock", "item"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DocKind {
    Scene,
    Encounter,
    Statblock,
    Item,
}

impl DocKind {
    pub fn parse(s: &str) -> Result<DocKind, String> {
        match s {
            "scene" => Ok(DocKind::Scene),
            "encounter" => Ok(DocKind::Encounter),
            "statblock" => Ok(DocKind::Statblock),
            "item" => Ok(DocKind::Item),
            other => Err(format!(
                "Unknown document type “{other}” (expected one of: {})",
                DOC_TYPES.join(", ")
            )),
        }
    }
}

/// Extra frontmatter keys we don't model — kept so round-trips are lossless.
pub type Custom = BTreeMap<String, Value>;

fn is_default<T: Default + PartialEq>(v: &T) -> bool {
    v == &T::default()
}

// ---------------------------------------------------------------------------
// Scene
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SceneMeta {
    #[serde(rename = "type")]
    pub doc_type: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub npcs: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_aloud: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(flatten, skip_serializing_if = "is_default")]
    pub custom: Custom,
}

// ---------------------------------------------------------------------------
// Encounter
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CombatantKind {
    #[default]
    Monster,
    Npc,
    Player,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Combatant {
    pub name: String,
    #[serde(default)]
    pub kind: CombatantKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hp: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ac: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initiative: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statblock: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(flatten, skip_serializing_if = "is_default")]
    pub custom: Custom,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EncounterMeta {
    #[serde(rename = "type")]
    pub doc_type: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub party_level: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub combatants: Vec<Combatant>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(flatten, skip_serializing_if = "is_default")]
    pub custom: Custom,
}

// ---------------------------------------------------------------------------
// Statblock
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StatblockKind {
    #[default]
    Monster,
    Npc,
    Player,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Abilities {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strength: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dexterity: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub constitution: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intelligence: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wisdom: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub charisma: Option<i64>,
    #[serde(flatten, skip_serializing_if = "is_default")]
    pub custom: Custom,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct NamedText {
    pub name: String,
    #[serde(default)]
    pub text: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StatblockMeta {
    #[serde(rename = "type")]
    pub doc_type: String,
    pub name: String,
    #[serde(default)]
    pub kind: StatblockKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hp: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ac: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initiative: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub speed: Option<String>,
    /// Free-form power rating 1–10 (no CR math; used by the encounter generator).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub power: Option<u8>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub abilities: Option<Abilities>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub traits: Vec<NamedText>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<NamedText>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reactions: Vec<NamedText>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(flatten, skip_serializing_if = "is_default")]
    pub custom: Custom,
}

// ---------------------------------------------------------------------------
// Item
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ItemMeta {
    #[serde(rename = "type")]
    pub doc_type: String,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rarity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(flatten, skip_serializing_if = "is_default")]
    pub custom: Custom,
}

// ---------------------------------------------------------------------------
// Parsed document
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Doc {
    pub rel_path: String,
    pub kind: DocKind,
    pub body: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scene: Option<SceneMeta>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encounter: Option<EncounterMeta>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub statblock: Option<StatblockMeta>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub item: Option<ItemMeta>,
}

/// Split `---` fenced YAML frontmatter from the body.
/// Returns `(yaml, body)`; the caller must handle `yaml == None`.
pub fn split_frontmatter(raw: &str) -> Result<(Option<String>, String), String> {
    let content = raw.trim_start_matches('\u{feff}');
    let mut lines = content.lines().peekable();

    // Skip blank lines before the opener.
    let mut opener_seen = false;
    for line in lines.by_ref() {
        if line.trim().is_empty() {
            continue;
        }
        if line.trim_end() == "---" {
            opener_seen = true;
        }
        break;
    }
    if !opener_seen {
        return Err("Missing frontmatter (expected a leading “---” block)".to_string());
    }

    let mut yaml_lines: Vec<&str> = Vec::new();
    let mut closed = false;
    for line in lines.by_ref() {
        if line.trim_end() == "---" {
            closed = true;
            break;
        }
        yaml_lines.push(line);
    }
    if !closed {
        return Err("Frontmatter block is not closed (missing closing “---”)".to_string());
    }

    let mut body_lines: Vec<&str> = lines.collect();
    while matches!(body_lines.first(), Some(l) if l.trim().is_empty()) {
        body_lines.remove(0);
    }
    while matches!(body_lines.last(), Some(l) if l.trim().is_empty()) {
        body_lines.pop();
    }
    let mut body = body_lines.join("\n");
    if !body.is_empty() {
        body.push('\n');
    }

    let yaml = if yaml_lines.iter().all(|l| l.trim().is_empty()) {
        return Err("Frontmatter block is empty".to_string());
    } else {
        Some(yaml_lines.join("\n"))
    };
    Ok((yaml, body))
}

/// Parse a raw file into a typed [`Doc`].
pub fn parse_doc(rel_path: &str, raw: &str) -> Result<Doc, String> {
    let (yaml, body) = split_frontmatter(raw)?;
    let yaml = yaml.ok_or_else(|| "Missing frontmatter".to_string())?;

    let value: Value =
        serde_yaml_ng::from_str(&yaml).map_err(|e| format!("Invalid YAML frontmatter: {e}"))?;
    let type_str = value
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Frontmatter is missing the “type” field".to_string())?;
    let kind = DocKind::parse(type_str)?;

    let mut doc = Doc {
        rel_path: rel_path.to_string(),
        kind,
        body,
        scene: None,
        encounter: None,
        statblock: None,
        item: None,
    };
    match kind {
        DocKind::Scene => {
            doc.scene = Some(serde_yaml_ng::from_str(&yaml).map_err(invalid)?);
        }
        DocKind::Encounter => {
            doc.encounter = Some(serde_yaml_ng::from_str(&yaml).map_err(invalid)?);
        }
        DocKind::Statblock => {
            doc.statblock = Some(serde_yaml_ng::from_str(&yaml).map_err(invalid)?);
        }
        DocKind::Item => {
            doc.item = Some(serde_yaml_ng::from_str(&yaml).map_err(invalid)?);
        }
    }
    Ok(doc)
}

fn invalid(e: serde_yaml_ng::Error) -> String {
    format!("Invalid frontmatter: {e}")
}

/// Render a document back to Markdown: typed frontmatter + body.
pub fn render_doc<T: Serialize>(meta: &T, body: &str) -> Result<String, String> {
    let yaml = serde_yaml_ng::to_string(meta).map_err(|e| e.to_string())?;
    // serde_yaml may or may not emit a leading "---" document marker; strip it.
    let yaml = yaml.strip_prefix("---\n").unwrap_or(&yaml);
    let yaml = yaml.trim_end();
    let body = if body.is_empty() || body.ends_with('\n') {
        body.to_string()
    } else {
        format!("{body}\n")
    };
    Ok(format!("---\n{}\n---\n{}", yaml, body))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::doc::{split_frontmatter, Custom};

    const SCENE: &str = "---\ntype: scene\ntitle: The Prancing Pony\ntags:\n  - tavern\n  - social\nnpcs:\n  - volo\nreadAloud: true\nstatus: ready\n---\n# The Prancing Pony\n\n> Read aloud text.\n";

    #[test]
    fn splits_frontmatter() {
        let (yaml, body) = split_frontmatter(SCENE).unwrap();
        let yaml = yaml.unwrap();
        assert!(yaml.contains("type: scene"));
        assert!(body.starts_with("# The Prancing Pony"));
        assert!(body.ends_with('\n') && !body.ends_with("\n\n"));
    }

    #[test]
    fn missing_frontmatter_is_an_error() {
        assert!(split_frontmatter("just body").is_err());
        assert!(split_frontmatter("---\ntype: scene\n").is_err()); // unclosed
        assert!(split_frontmatter("---\n---\nbody").is_err()); // empty
    }

    #[test]
    fn parses_scene() {
        let doc = parse_doc("scenes/pony.md", SCENE).unwrap();
        assert_eq!(doc.kind, DocKind::Scene);
        let scene = doc.scene.unwrap();
        assert_eq!(scene.title, "The Prancing Pony");
        assert_eq!(scene.tags, vec!["tavern", "social"]);
        assert_eq!(scene.npcs, vec!["volo"]);
        assert_eq!(scene.read_aloud, Some(true));
        assert_eq!(scene.status.as_deref(), Some("ready"));
    }

    #[test]
    fn scene_round_trips() {
        let doc = parse_doc("scenes/pony.md", SCENE).unwrap();
        let scene = doc.scene.clone().unwrap();
        let rendered = render_doc(&scene, &doc.body).unwrap();
        let reparsed = parse_doc("scenes/pony.md", &rendered).unwrap();
        assert_eq!(doc, reparsed);
    }

    #[test]
    fn unknown_keys_are_preserved() {
        let raw = "---\ntype: scene\ntitle: X\nweather: stormy\ncustomBits:\n  a: 1\n---\nbody\n";
        let doc = parse_doc("scenes/x.md", raw).unwrap();
        let scene = doc.scene.unwrap();
        let custom: &Custom = &scene.custom;
        assert!(custom.contains_key("weather"));
        assert!(custom.contains_key("customBits"));
        let rendered = render_doc(&scene, &doc.body).unwrap();
        assert!(rendered.contains("weather: stormy"));
    }

    #[test]
    fn encounter_with_combatants_round_trips() {
        let raw = "---\ntype: encounter\ntitle: Goblin Ambush\npartyLevel: 3\ncombatants:\n  - name: Grashnak\n    kind: npc\n    hp: 45\n    ac: 15\n    initiative: 2\n    statblock: ../npcs/grashnak.md\n  - name: Goblin A\n    hp: 7\n    ac: 15\n---\n# Setup\n";
        let doc = parse_doc("encounters/ambush.md", raw).unwrap();
        let meta = doc.encounter.clone().unwrap();
        assert_eq!(meta.party_level, Some(3));
        assert_eq!(meta.combatants.len(), 2);
        assert_eq!(meta.combatants[0].kind, CombatantKind::Npc);
        assert_eq!(meta.combatants[1].kind, CombatantKind::Monster); // default
        assert_eq!(meta.combatants[0].statblock.as_deref(), Some("../npcs/grashnak.md"));

        let rendered = render_doc(&meta, &doc.body).unwrap();
        let reparsed = parse_doc("encounters/ambush.md", &rendered).unwrap();
        assert_eq!(doc, reparsed);
    }

    #[test]
    fn statblock_round_trips() {
        let raw = "---\ntype: statblock\nname: Goblin\nkind: monster\nhp: 7\nac: 15\ninitiative: 2\nspeed: 30 ft\npower: 1\nabilities:\n  strength: 8\n  dexterity: 14\ntraits:\n  - name: Nimble Escape\n    text: Can Disengage or Hide as a bonus action.\nactions:\n  - name: Scimitar\n    text: \"Melee: +4 to hit, 1d6+2 slashing.\"\ntags:\n  - goblinoid\n---\nFree-form notes.\n";
        let doc = parse_doc("statblocks/goblin.md", raw).unwrap();
        let meta = doc.statblock.clone().unwrap();
        let ab = meta.abilities.as_ref().unwrap();
        assert_eq!(ab.strength, Some(8));
        assert_eq!(ab.dexterity, Some(14));
        assert_eq!(ab.constitution, None);
        assert_eq!(meta.actions[0].name, "Scimitar");

        let rendered = render_doc(&meta, &doc.body).unwrap();
        let reparsed = parse_doc("statblocks/goblin.md", &rendered).unwrap();
        assert_eq!(doc, reparsed);
    }

    #[test]
    fn item_round_trips() {
        let raw = "---\ntype: item\nname: Cloak of Embers\nrarity: uncommon\nkind: wondrous\nvalue: 250 gp\ntags:\n  - magic\n---\nWarm to the touch.\n";
        let doc = parse_doc("items/cloak.md", raw).unwrap();
        let meta = doc.item.clone().unwrap();
        assert_eq!(meta.rarity.as_deref(), Some("uncommon"));
        let rendered = render_doc(&meta, &doc.body).unwrap();
        let reparsed = parse_doc("items/cloak.md", &rendered).unwrap();
        assert_eq!(doc, reparsed);
    }

    #[test]
    fn rejects_unknown_and_missing_types() {
        assert!(parse_doc("x.md", "---\ntype: spell\ntitle: X\n---\n").is_err());
        assert!(parse_doc("x.md", "---\ntitle: X\n---\n").is_err());
    }

    #[test]
    fn normalizes_body_trailing_whitespace() {
        let raw = "---\ntype: item\nname: X\n---\nbody\n\n\n";
        let doc = parse_doc("i.md", raw).unwrap();
        assert_eq!(doc.body, "body\n");
    }
}
