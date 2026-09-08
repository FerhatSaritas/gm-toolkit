import { listGeneratorPacks } from "../commands";
import type { PackInfo } from "../commands";

export const forge = $state<{ packs: PackInfo[]; loading: boolean }>({
  packs: [],
  loading: false
});

export async function refreshPacks() {
  forge.loading = true;
  try {
    forge.packs = await listGeneratorPacks();
  } catch (e) {
    console.error(e);
  } finally {
    forge.loading = false;
  }
}
