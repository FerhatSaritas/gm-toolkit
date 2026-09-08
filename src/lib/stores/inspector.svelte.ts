import { openStatblock } from "../commands";
import type { Doc } from "../commands";
import { notify } from "./campaign.svelte";

export const inspector = $state<{
  statblock: Doc | null;
  loading: boolean;
}>({
  statblock: null,
  loading: false
});

export async function showStatblock(basePath: string | null, link: string) {
  try {
    inspector.loading = true;
    inspector.statblock = await openStatblock(basePath, link);
  } catch (e) {
    notify(String(e), "error");
  } finally {
    inspector.loading = false;
  }
}

export function closeStatblock() {
  inspector.statblock = null;
}
