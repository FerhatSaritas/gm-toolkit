//! Live encounter state — the *run-time* counterpart of encounter blueprints.
//!
//! Blueprints (Markdown) describe what the encounter looks like at rest;
//! this module persists what is happening at the table (rolled initiative,
//! current HP, conditions, turn pointer) as JSON under `<campaign>/.state/`,
//! mirroring the blueprint's folder. The Markdown file is never touched
//! during play.

use crate::campaign::slugify;
use crate::doc::{self, CombatantKind, DocKind};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CombatantState {
    pub id: String,
    pub name: String,
    pub kind: CombatantKind,
    /// Rolled (or manually entered) initiative total.
    #[serde(default)]
    pub initiative: Option<i64>,
    /// Blueprint modifier used when auto-rolling.
    #[serde(default)]
    pub initiative_mod: Option<i64>,
    pub hp: i64,
    #[serde(default)]
    pub max_hp: Option<i64>,
    #[serde(default)]
    pub temp_hp: i64,
    #[serde(default)]
    pub ac: Option<i64>,
    #[serde(default)]
    pub statblock: Option<String>,
    #[serde(default)]
    pub conditions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EncounterState {
    /// Blueprint this state belongs to (campaign-relative).
    pub encounter_path: String,
    #[serde(default = "default_round")]
    pub round: u32,
    #[serde(default)]
    pub turn_id: Option<String>,
    pub combatants: Vec<CombatantState>,
}

fn default_round() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EncounterLoad {
    pub state: EncounterState,
    pub restored: bool,
}

/// `encounters/foo.md` → `.state/encounters/foo.json`
fn state_path_for(campaign_root: &Path, rel_path: &str) -> PathBuf {
    let rel = Path::new(rel_path);
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

fn unique_slug(name: &str, used: &mut Vec<String>) -> String {
    let base = slugify(name);
    let mut candidate = base.clone();
    let mut n = 2;
    while used.iter().any(|u| u == &candidate) {
        candidate = format!("{base}-{n}");
        n += 1;
    }
    used.push(candidate.clone());
    candidate
}

/// Build a fresh state from the blueprint on disk.
pub fn fresh_state(campaign_root: &Path, rel_path: &str) -> Result<EncounterState, String> {
    let raw = fs::read_to_string(campaign_root.join(rel_path))
        .map_err(|e| format!("Cannot read encounter blueprint: {e}"))?;
    let parsed = doc::parse_doc(rel_path, &raw)?;
    if parsed.kind != DocKind::Encounter {
        return Err("Not an encounter blueprint".to_string());
    }
    let meta = parsed
        .encounter
        .ok_or_else(|| "Not an encounter blueprint".to_string())?;

    let mut used: Vec<String> = Vec::new();
    let combatants = meta
        .combatants
        .iter()
        .map(|c| {
            let hp = c.hp.unwrap_or(0);
            CombatantState {
                id: unique_slug(&c.name, &mut used),
                name: c.name.clone(),
                kind: c.kind,
                initiative: None,
                initiative_mod: c.initiative,
                hp,
                max_hp: c.hp,
                temp_hp: 0,
                ac: c.ac,
                statblock: c.statblock.clone(),
                conditions: Vec::new(),
            }
        })
        .collect();

    Ok(EncounterState {
        encounter_path: rel_path.to_string(),
        round: 1,
        turn_id: None,
        combatants,
    })
}

/// Start (or resume) an encounter: returns the persisted state if one exists,
/// otherwise builds a fresh one from the blueprint and persists it.
pub fn start_encounter_at(campaign_root: &Path, rel_path: &str) -> Result<EncounterLoad, String> {
    let path = state_path_for(campaign_root, rel_path);
    if path.is_file() {
        let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let state: EncounterState = serde_json::from_str(&raw)
            .map_err(|e| format!("Corrupt encounter state: {e}"))?;
        return Ok(EncounterLoad {
            state,
            restored: true,
        });
    }
    let state = fresh_state(campaign_root, rel_path)?;
    save_state_at(campaign_root, &state)?;
    Ok(EncounterLoad {
        state,
        restored: false,
    })
}

pub fn save_state_at(campaign_root: &Path, state: &EncounterState) -> Result<(), String> {
    // The blueprint path is re-validated here; the state path is derived from
    // it, so a traversal attempt in encounter_path cannot escape `.state/`.
    crate::ensure_in_campaign(campaign_root, &state.encounter_path)?;
    let path = state_path_for(campaign_root, &state.encounter_path);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let raw = serde_json::to_string_pretty(state).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, raw).map_err(|e| e.to_string())?;
    fs::rename(&tmp, &path).map_err(|e| e.to_string())
}

/// Discard the live state and rebuild from the blueprint.
pub fn reset_encounter_at(campaign_root: &Path, rel_path: &str) -> Result<EncounterState, String> {
    let path = state_path_for(campaign_root, rel_path);
    if path.is_file() {
        fs::remove_file(&path).map_err(|e| e.to_string())?;
    }
    let state = fresh_state(campaign_root, rel_path)?;
    save_state_at(campaign_root, &state)?;
    Ok(state)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BLUEPRINT: &str = "---\ntype: encounter\ntitle: Ambush\npartyLevel: 3\ncombatants:\n  - name: Grashnak\n    kind: npc\n    hp: 45\n    ac: 15\n    initiative: 2\n    statblock: ../npcs/grashnak.md\n  - name: Goblin A\n    hp: 7\n    ac: 15\n  - name: Goblin A\n    hp: 7\n---\n# Setup\n";

    fn setup_campaign() -> (tempdir::TempDir, String) {
        let dir = tempdir::TempDir::new("gmtest").unwrap();
        std::fs::create_dir_all(dir.path().join("encounters")).unwrap();
        std::fs::write(dir.path().join("encounters/ambush.md"), BLUEPRINT).unwrap();
        (dir, "encounters/ambush.md".to_string())
    }

    // Minimal RAII temp dir (avoid adding a dev-dependency).
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
    fn builds_fresh_state_from_blueprint() {
        let (dir, rel) = setup_campaign();
        let state = fresh_state(dir.path(), &rel).unwrap();
        assert_eq!(state.combatants.len(), 3);
        assert_eq!(state.combatants[0].id, "grashnak");
        assert_eq!(state.combatants[0].hp, 45);
        assert_eq!(state.combatants[0].max_hp, Some(45));
        assert_eq!(state.combatants[0].initiative_mod, Some(2));
        assert_eq!(state.combatants[0].statblock.as_deref(), Some("../npcs/grashnak.md"));
        assert_eq!(state.combatants[1].id, "goblin-a");
        assert_eq!(state.combatants[2].id, "goblin-a-2"); // deduped
        assert_eq!(state.round, 1);
        assert!(state.turn_id.is_none());
    }

    #[test]
    fn start_resumes_persisted_state() {
        let (dir, rel) = setup_campaign();
        let first = start_encounter_at(dir.path(), &rel).unwrap();
        assert!(!first.restored);

        let mut mutated = first.state.clone();
        mutated.combatants[0].hp = 12;
        mutated.combatants[0].conditions.push("prone".into());
        mutated.round = 3;
        save_state_at(dir.path(), &mutated).unwrap();

        let second = start_encounter_at(dir.path(), &rel).unwrap();
        assert!(second.restored);
        assert_eq!(second.state, mutated);
    }

    #[test]
    fn reset_rebuilds_from_blueprint() {
        let (dir, rel) = setup_campaign();
        let first = start_encounter_at(dir.path(), &rel).unwrap().state;
        let mut mutated = first.clone();
        mutated.round = 9;
        mutated.combatants[0].hp = 0;
        save_state_at(dir.path(), &mutated).unwrap();

        let fresh = reset_encounter_at(dir.path(), &rel).unwrap();
        assert_eq!(fresh, first);
        assert!(fresh.turn_id.is_none());
    }

    #[test]
    fn state_file_mirrors_blueprint_folder() {
        let (dir, rel) = setup_campaign();
        let _ = start_encounter_at(dir.path(), &rel).unwrap();
        assert!(dir.path().join(".state/encounters/ambush.json").is_file());
    }

    #[test]
    fn slugify_and_uniqueness() {
        assert_eq!(slugify("Goblin Archer #3"), "goblin-archer-3");
        assert_eq!(slugify("  --  "), "combatant");
        let mut used = vec!["goblin".to_string()];
        assert_eq!(unique_slug("Goblin", &mut used), "goblin-2");
        assert_eq!(unique_slug("!!!", &mut used), "combatant");
        assert_eq!(unique_slug("!!!", &mut used), "combatant-2");
    }

    #[test]
    fn rejects_non_encounter_and_missing_files() {
        let (dir, _rel) = setup_campaign();
        std::fs::write(dir.path().join("encounters/scene.md"), "---\ntype: scene\ntitle: X\n---\n").unwrap();
        assert!(fresh_state(dir.path(), "encounters/scene.md").is_err());
        assert!(fresh_state(dir.path(), "encounters/nope.md").is_err());
    }
}
