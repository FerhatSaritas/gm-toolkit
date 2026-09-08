export type ViewKind = "library" | "creatures" | "generators";
export type CreatureFilter = "all" | "monster" | "npc" | "player";

export const ui = $state<{
  view: ViewKind;
  creatureFilter: CreatureFilter;
  helpOpen: boolean;
}>({
  view: "library",
  creatureFilter: "all",
  helpOpen: false
});

export const showHelp = $state<{ open: boolean }>({ open: false });

export function toggleHelp() {
  showHelp.open = !showHelp.open;
}

export function showLibrary() {
  ui.view = "library";
}

export function showBestiary() {
  ui.view = "creatures";
  ui.creatureFilter = "all";
}

export function showGallery() {
  ui.view = "creatures";
  ui.creatureFilter = "npc";
}

export function showForge() {
  ui.view = "generators";
}
