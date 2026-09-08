import { searchCampaign } from "../commands";
import type { SearchHit } from "../commands";
import { openRelPath } from "./campaign.svelte";
import { showStatblock } from "./inspector.svelte";
import { notify } from "./campaign.svelte";

export const palette = $state<{
  open: boolean;
  query: string;
  results: SearchHit[];
  selected: number;
}>({
  open: false,
  query: "",
  results: [],
  selected: 0
});

let debounceTimer: ReturnType<typeof setTimeout> | undefined;
let searchSeq = 0;

export function openPalette() {
  palette.open = true;
  palette.query = "";
  palette.results = [];
  palette.selected = 0;
}

export function closePalette() {
  palette.open = false;
}

export function togglePalette() {
  if (palette.open) closePalette();
  else openPalette();
}

export function setQuery(value: string) {
  palette.query = value;
  palette.selected = 0;
  if (debounceTimer) clearTimeout(debounceTimer);
  const seq = ++searchSeq;
  const q = value.trim();
  if (!q) {
    palette.results = [];
    return;
  }
  debounceTimer = setTimeout(async () => {
    try {
      const results = await searchCampaign(q);
      if (seq === searchSeq) {
        palette.results = results;
        palette.selected = 0;
      }
    } catch (e) {
      notify(String(e), "error");
    }
  }, 120);
}

export function moveSelection(delta: 1 | -1) {
  if (palette.results.length === 0) return;
  palette.selected = Math.max(
    0,
    Math.min(palette.results.length - 1, palette.selected + delta)
  );
}

export function chooseSelected() {
  const hit = palette.results[palette.selected];
  closePalette();
  if (!hit) return;
  openHit(hit);
}

export function openHit(hit: SearchHit) {
  if (hit.kind === "statblock") {
    showStatblock(null, hit.relPath);
  } else {
    void openRelPath(hit.relPath);
  }
}
