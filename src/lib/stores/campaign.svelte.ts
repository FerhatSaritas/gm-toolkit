import { chooseCampaign, getConfig, loadDoc, readCampaignTree, renderMarkdown } from "../commands";
import type { CampaignInfo, Doc, TreeNode } from "../commands";
import { showLibrary } from "./ui.svelte";

export const campaign = $state<{
  info: CampaignInfo | null;
  tree: TreeNode[];
  doc: Doc | null;
  loading: boolean;
}>({
  info: null,
  tree: [],
  doc: null,
  loading: false
});

export const toast = $state<{ message: string; kind: "info" | "error" }>({
  message: "",
  kind: "info"
});

let toastTimer: ReturnType<typeof setTimeout> | undefined;

export function notify(message: string, kind: "info" | "error" = "info") {
  toast.message = message;
  toast.kind = kind;
  if (toastTimer) clearTimeout(toastTimer);
  toastTimer = setTimeout(() => (toast.message = ""), 3200);
}

export async function initCampaign() {
  campaign.loading = true;
  try {
    const info = await getConfig();
    if (info) {
      campaign.info = info;
      campaign.tree = await readCampaignTree();
    }
  } catch (e) {
    notify(String(e), "error");
  } finally {
    campaign.loading = false;
  }
}

export async function openCampaign() {
  campaign.loading = true;
  try {
    const info = await chooseCampaign();
    if (info) {
      campaign.info = info;
      campaign.doc = null;
      campaign.tree = await readCampaignTree();
      notify(`Campaign “${info.name}” opened`);
    }
  } catch (e) {
    notify(String(e), "error");
  } finally {
    campaign.loading = false;
  }
}

export async function openRelPath(relPath: string) {
  try {
    const doc = await loadDoc(relPath);
    doc.bodyHtml = await renderMarkdown(doc.body);
    campaign.doc = doc;
    showLibrary();
  } catch (e) {
    campaign.doc = null;
    notify(String(e), "error");
  }
}

export async function selectFile(node: TreeNode) {
  if (node.kind !== "file") return;
  await openRelPath(node.relPath);
}
