mod campaign;
mod catalog;
mod config;
mod dice;
mod doc;
mod encounter;
mod generator;
mod import;
mod markdown;
mod search;
mod statblock;

use campaign::{read_tree, CampaignInfo, TreeNode};
use config::{load_config, save_config, Config};
use doc::{Doc, DocKind, EncounterMeta, ItemMeta, SceneMeta, StatblockMeta};
use search::SearchHit;
use std::{
    fs,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};
use notify::Watcher;
use tauri::State;

#[derive(Default)]
struct AppState {
    campaign_root: Mutex<Option<PathBuf>>,
    index: Mutex<Option<Vec<search::SearchDoc>>>,
    index_dirty: Arc<AtomicBool>,
    watcher: Mutex<Option<notify::RecommendedWatcher>>,
}

impl AppState {
    fn mark_index_dirty(&self) {
        self.index_dirty.store(true, Ordering::Relaxed);
    }

    /// Interior-mutable reset: new index slot + fresh file watcher.
    fn reset_index(&self, root: &Path) {
        *self.index.lock().unwrap() = None;
        self.mark_index_dirty();
        let dirty = self.index_dirty.clone();
        let mut slot = self.watcher.lock().unwrap();
        *slot = None; // drop the previous watcher
        match watch_campaign(root, dirty) {
            Ok(w) => *slot = Some(w),
            Err(e) => eprintln!("file watching unavailable: {e}"),
        }
    }
}

fn watch_campaign(
    root: &Path,
    dirty: Arc<AtomicBool>,
) -> notify::Result<notify::RecommendedWatcher> {
    let flag = dirty.clone();
    let mut watcher = notify::recommended_watcher(
        move |res: Result<notify::Event, notify::Error>| {
            let Ok(ev) = res else { return };
            // Live encounter state churns constantly — never dirty the index for it.
            if ev
                .paths
                .iter()
                .any(|p| p.components().any(|c| c.as_os_str() == ".state"))
            {
                return;
            }
            flag.store(true, Ordering::Relaxed);
        },
    )?;
    watcher.watch(root, notify::RecursiveMode::Recursive)?;
    Ok(watcher)
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct FileDoc {
    path: String,
    name: String,
    content: String,
}

fn open_campaign_at(state: &AppState, app: &tauri::AppHandle, dir: &Path) -> Result<CampaignInfo, String> {
    if !dir.is_dir() {
        return Err(format!("Not a directory: {}", dir.display()));
    }
    let dir = dir
        .canonicalize()
        .map_err(|e| format!("Cannot resolve directory: {e}"))?;
    let info = CampaignInfo::from_dir(&dir).map_err(|e| e.to_string())?;
    save_config(
        app,
        &Config {
            campaign_path: Some(dir.clone()),
        },
    )
    .map_err(|e| e.to_string())?;
    *state.campaign_root.lock().unwrap() = Some(dir.clone());
    state.reset_index(&dir);
    Ok(info)
}

#[tauri::command]
fn get_config(state: State<AppState>, app: tauri::AppHandle) -> Result<Option<CampaignInfo>, String> {
    let cfg = load_config(&app).map_err(|e| e.to_string())?;
    match cfg.campaign_path {
        Some(p) if p.is_dir() => {
            let info = CampaignInfo::from_dir(&p).map_err(|e| e.to_string())?;
            *state.campaign_root.lock().unwrap() = Some(p.clone());
            state.reset_index(&p);
            Ok(Some(info))
        }
        Some(_) => {
            // Stale path: silently clear it.
            let _ = save_config(&app, &Config { campaign_path: None });
            Ok(None)
        }
        None => Ok(None),
    }
}

#[tauri::command]
fn choose_campaign(
    state: State<AppState>,
    app: tauri::AppHandle,
) -> Result<Option<CampaignInfo>, String> {
    use tauri_plugin_dialog::DialogExt;

    let picked = app
        .dialog()
        .file()
        .set_title("Open Campaign Folder")
        .blocking_pick_folder();

    let Some(picked) = picked else {
        return Ok(None);
    };
    let dir = picked
        .as_path()
        .map(Path::to_path_buf)
        .ok_or_else(|| "Selection is not a filesystem path".to_string())?;
    open_campaign_at(&state, &app, &dir).map(Some)
}

#[tauri::command]
fn set_campaign(
    state: State<AppState>,
    app: tauri::AppHandle,
    path: String,
) -> Result<CampaignInfo, String> {
    open_campaign_at(&state, &app, Path::new(&path))
}

#[tauri::command]
fn read_campaign_tree(state: State<AppState>) -> Result<Vec<TreeNode>, String> {
    let root = state
        .campaign_root
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "No campaign open".to_string())?;
    read_tree(&root).map_err(|e| e.to_string())
}

/// Lexical + canonical validation that `rel_path` stays inside `campaign_root`.
/// The target must exist. Public so `encounter.rs` can validate state paths.
pub fn ensure_in_campaign(campaign_root: &Path, rel_path: &str) -> Result<PathBuf, String> {
    if rel_path.starts_with('/')
        || rel_path.starts_with('\\')
        || Path::new(rel_path)
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err("Path escapes the campaign folder".to_string());
    }
    let canonical_root = campaign_root
        .canonicalize()
        .map_err(|e| format!("Campaign folder unavailable: {e}"))?;
    let canonical = campaign_root
        .join(rel_path)
        .canonicalize()
        .map_err(|e| format!("File not found: {e}"))?;
    if !canonical.starts_with(&canonical_root) {
        return Err("Path escapes the campaign folder".to_string());
    }
    Ok(canonical)
}

/// Resolve `rel_path` inside the open campaign, refusing path traversal.
fn resolve_in_campaign(state: &AppState, rel_path: &str) -> Result<(PathBuf, PathBuf), String> {
    let root = state
        .campaign_root
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "No campaign open".to_string())?;
    let canonical = ensure_in_campaign(&root, rel_path)?;
    let canonical_root = root
        .canonicalize()
        .map_err(|e| format!("Campaign folder unavailable: {e}"))?;
    Ok((canonical_root, canonical))
}

#[tauri::command]
fn read_file(state: State<AppState>, rel_path: String) -> Result<FileDoc, String> {
    let (_, path) = resolve_in_campaign(&state, &rel_path)?;
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let name = path
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_default();
    Ok(FileDoc {
        path: rel_path,
        name,
        content,
    })
}

#[tauri::command]
fn load_doc(state: State<AppState>, rel_path: String) -> Result<Doc, String> {
    let (_, path) = resolve_in_campaign(&state, &rel_path)?;
    let raw = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    doc::parse_doc(&rel_path, &raw)
}

/// Exactly one metadata payload must be provided; it selects the doc type.
#[tauri::command]
fn save_doc(
    state: State<AppState>,
    rel_path: String,
    body: String,
    scene: Option<SceneMeta>,
    encounter: Option<EncounterMeta>,
    statblock: Option<StatblockMeta>,
    item: Option<ItemMeta>,
) -> Result<DocKind, String> {
    let provided: [(&str, bool); 4] = [
        ("scene", scene.is_some()),
        ("encounter", encounter.is_some()),
        ("statblock", statblock.is_some()),
        ("item", item.is_some()),
    ];
    let mut chosen: Option<DocKind> = None;
    for (name, present) in provided {
        if present {
            if chosen.is_some() {
                return Err("Provide exactly one document payload".to_string());
            }
            chosen = Some(doc::DocKind::parse(name).unwrap());
        }
    }
    let kind = chosen.ok_or_else(|| "Provide exactly one document payload".to_string())?;

    let content = match (&scene, &encounter, &statblock, &item) {
        (Some(m), _, _, _) => doc::render_doc(&SceneMeta { doc_type: "scene".into(), ..m.clone() }, &body),
        (_, Some(m), _, _) => doc::render_doc(&EncounterMeta { doc_type: "encounter".into(), ..m.clone() }, &body),
        (_, _, Some(m), _) => doc::render_doc(&StatblockMeta { doc_type: "statblock".into(), ..m.clone() }, &body),
        (_, _, _, Some(m)) => doc::render_doc(&ItemMeta { doc_type: "item".into(), ..m.clone() }, &body),
        _ => unreachable!(),
    }?;

    let (_, path) = resolve_in_campaign(&state, &rel_path)?;
    fs::write(&path, content).map_err(|e| e.to_string())?;
    state.mark_index_dirty();
    Ok(kind)
}

#[tauri::command]
fn save_file(state: State<AppState>, rel_path: String, content: String) -> Result<(), String> {
    let (_, path) = resolve_in_campaign(&state, &rel_path)?;
    fs::write(&path, content).map_err(|e| e.to_string())
}

#[tauri::command]
fn render_markdown(md: String) -> Result<String, String> {
    Ok(markdown::render(&md))
}

#[tauri::command]
fn start_encounter(state: State<AppState>, rel_path: String) -> Result<encounter::EncounterLoad, String> {
    let root = campaign_root(&state)?;
    encounter::start_encounter_at(&root, &rel_path)
}

#[tauri::command]
fn save_encounter_state(
    state: State<AppState>,
    encounter_state: encounter::EncounterState,
) -> Result<(), String> {
    let root = campaign_root(&state)?;
    encounter::save_state_at(&root, &encounter_state)
}

#[tauri::command]
fn reset_encounter(state: State<AppState>, rel_path: String) -> Result<encounter::EncounterState, String> {
    let root = campaign_root(&state)?;
    encounter::reset_encounter_at(&root, &rel_path)
}

#[tauri::command]
fn roll_dice(expr: String) -> Result<dice::RollResult, String> {
    dice::roll(&expr)
}

#[tauri::command]
fn open_statblock(
    state: State<AppState>,
    base_path: Option<String>,
    link: String,
) -> Result<Doc, String> {
    let root = campaign_root(&state)?;
    statblock::open_at(&root, base_path.as_deref(), &link)
}

#[tauri::command]
fn list_statblocks(state: State<AppState>) -> Result<Vec<catalog::StatblockSummary>, String> {
    let root = campaign_root(&state)?;
    catalog::list_statblocks(&root)
}

#[tauri::command]
fn list_encounters(state: State<AppState>) -> Result<Vec<catalog::EncounterSummary>, String> {
    let root = campaign_root(&state)?;
    catalog::list_encounters(&root)
}

#[tauri::command]
fn add_combatant_to_encounter(
    state: State<AppState>,
    encounter_path: String,
    combatant: doc::Combatant,
) -> Result<catalog::EncounterSummary, String> {
    let root = campaign_root(&state)?;
    catalog::add_combatant_to_encounter(&root, &encounter_path, &combatant)
}

#[tauri::command]
fn create_encounter(
    state: State<AppState>,
    title: String,
    combatant: Option<doc::Combatant>,
) -> Result<String, String> {
    let root = campaign_root(&state)?;
    catalog::create_encounter(&root, &title, combatant.as_ref())
}

#[tauri::command]
fn create_statblock(
    state: State<AppState>,
    name: String,
    kind: doc::StatblockKind,
    hp: Option<i64>,
    ac: Option<i64>,
    power: Option<u8>,
) -> Result<String, String> {
    let root = campaign_root(&state)?;
    catalog::create_statblock(&root, &name, kind, hp, ac, power)
}

#[tauri::command]
fn duplicate_file(state: State<AppState>, rel_path: String) -> Result<String, String> {
    let root = campaign_root(&state)?;
    catalog::duplicate_file(&root, &rel_path)
}

#[tauri::command]
fn delete_file(state: State<AppState>, rel_path: String) -> Result<(), String> {
    let root = campaign_root(&state)?;
    catalog::delete_file(&root, &rel_path)
}

#[tauri::command]
fn pick_import_file(app: tauri::AppHandle) -> Option<String> {
    use tauri_plugin_dialog::DialogExt;
    let picked = app
        .dialog()
        .file()
        .set_title("Import Statblock (.md / .json)")
        .add_filter("Statblock", &["md", "json"])
        .blocking_pick_file();
    picked.and_then(|p| p.as_path().map(|x| x.display().to_string()))
}

#[tauri::command]
fn import_preview(path: String) -> Result<import::ImportPreview, String> {
    import::preview(&path)
}

#[tauri::command]
fn commit_import(
    state: State<AppState>,
    statblock: doc::StatblockMeta,
    body: String,
) -> Result<String, String> {
    let root = campaign_root(&state)?;
    import::commit(&root, &statblock, &body)
}

#[tauri::command]
fn search_campaign(state: State<AppState>, query: String) -> Result<Vec<SearchHit>, String> {
    let root = campaign_root(&state)?;
    search::search(&root, &state.index, &state.index_dirty, &query)
}

#[tauri::command]
fn list_generator_packs(state: State<AppState>) -> Result<Vec<generator::PackInfo>, String> {
    let root = campaign_root(&state)?;
    Ok(generator::list_packs(&root))
}

#[tauri::command]
fn generate_from_pack(
    state: State<AppState>,
    pack_id: String,
    template: Option<String>,
) -> Result<generator::Generated, String> {
    let root = campaign_root(&state)?;
    generator::generate(&root, &pack_id, template.as_deref())
}

#[tauri::command]
fn generate_encounter_draft(
    state: State<AppState>,
    party_level: u32,
    band: u32,
    count: usize,
) -> Result<Vec<catalog::StatblockSummary>, String> {
    let root = campaign_root(&state)?;
    generator::generate_encounter_draft(&root, party_level, band, count)
}

#[tauri::command]
fn save_generated_statblock(
    state: State<AppState>,
    statblock: StatblockMeta,
    body: String,
) -> Result<String, String> {
    let root = campaign_root(&state)?;
    let rel = generator::save_generated_statblock(&root, &statblock, &body)?;
    state.mark_index_dirty();
    Ok(rel)
}

#[tauri::command]
fn save_generated_item(
    state: State<AppState>,
    item: ItemMeta,
    body: String,
) -> Result<String, String> {
    let root = campaign_root(&state)?;
    let rel = generator::save_generated_item(&root, &item, &body)?;
    state.mark_index_dirty();
    Ok(rel)
}

#[tauri::command]
fn save_generated_encounter(
    state: State<AppState>,
    title: String,
    party_level: Option<u32>,
    combatants: Vec<doc::Combatant>,
    body: String,
) -> Result<String, String> {
    let root = campaign_root(&state)?;
    let rel = generator::save_encounter_draft(&root, &title, party_level, &combatants, &body)?;
    state.mark_index_dirty();
    Ok(rel)
}

fn campaign_root(state: &AppState) -> Result<PathBuf, String> {
    state
        .campaign_root
        .lock()
        .unwrap()
        .clone()
        .ok_or_else(|| "No campaign open".to_string())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            get_config,
            choose_campaign,
            set_campaign,
            read_campaign_tree,
            read_file,
            load_doc,
            save_doc,
            render_markdown,
            start_encounter,
            save_encounter_state,
            reset_encounter,
            roll_dice,
            open_statblock,
            list_statblocks,
            list_encounters,
            add_combatant_to_encounter,
            create_encounter,
            create_statblock,
            duplicate_file,
            delete_file,
            pick_import_file,
            import_preview,
            commit_import,
            search_campaign,
            list_generator_packs,
            generate_from_pack,
            generate_encounter_draft,
            save_generated_statblock,
            save_generated_item,
            save_generated_encounter,
            save_file
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
