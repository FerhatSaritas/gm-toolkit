import {
  listEncounters,
  listStatblocks
} from "../commands";
import type { EncounterSummary, StatblockSummary } from "../commands";

export const catalog = $state<{
  creatures: StatblockSummary[];
  encounters: EncounterSummary[];
  loading: boolean;
}>({
  creatures: [],
  encounters: [],
  loading: false
});

export async function refreshCatalog() {
  catalog.loading = true;
  try {
    const [creatures, encounters] = await Promise.all([listStatblocks(), listEncounters()]);
    catalog.creatures = creatures;
    catalog.encounters = encounters;
  } catch (e) {
    console.error(e);
  } finally {
    catalog.loading = false;
  }
}
