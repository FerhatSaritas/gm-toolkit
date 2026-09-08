<script lang="ts">
  import type { Doc } from "../commands";

  interface Section {
    id: number;
    level: number;
    title: string;
    html: string;
    privileged: boolean;
    readAloud: boolean;
  }

  let {
    doc,
    headingToHide = ""
  }: { doc: Doc; headingToHide?: string } = $props();

  let focus = $state(false);
  let collapsed = $state<Record<number, boolean>>({});

  const bodyHtml = $derived(doc.bodyHtml ?? "");

  interface Split {
    intro: string;
    introReadAloud: boolean;
    sections: Section[];
  }

  function buildSections(html: string, hideHeading: string): Split {
    const parsed = new DOMParser().parseFromString(html, "text/html");
    const children = Array.from(parsed.body.children);

    const normalize = (s: string) => s.trim().replace(/\s+/g, " ").toLowerCase();

    // Content before the first heading is the scene intro.
    let i = 0;
    const introNodes: string[] = [];
    while (i < children.length && !/^H[1-3]$/.test(children[i].tagName)) {
      introNodes.push(children[i].outerHTML);
      i++;
    }
    // A first-level heading that repeats the header title is redundant.
    if (
      i < children.length &&
      children[i].tagName === "H1" &&
      normalize(children[i].textContent ?? "") === normalize(hideHeading)
    ) {
      i++;
    }

    const sections: Section[] = [];
    let id = 0;
    while (i < children.length) {
      const head = children[i];
      if (!/^H[1-4]$/.test(head.tagName)) {
        // Unheaded content after the intro — treat as a titleless section.
        const parts: string[] = [];
        while (i < children.length && !/^H[1-4]$/.test(children[i].tagName)) {
          parts.push(children[i].outerHTML);
          i++;
        }
        sections.push({
          id: id++,
          level: 4,
          title: "",
          html: parts.join("\n"),
          privileged: false,
          readAloud: parts.join("\n").includes("<blockquote")
        });
        continue;
      }
      const level = Number(head.tagName[1]);
      const title = (head.textContent ?? "").trim();
      const parts: string[] = [];
      i++;
      while (i < children.length && !/^H[1-4]$/.test(children[i].tagName)) {
        parts.push(children[i].outerHTML);
        i++;
      }
      sections.push({
        id: id++,
        level,
        title,
        html: parts.join("\n"),
        privileged: isPrivileged(title),
        readAloud: parts.join("\n").includes("<blockquote")
      });
    }
    return {
      intro: introNodes.join("\n"),
      introReadAloud: introNodes.join("\n").includes("<blockquote"),
      sections
    };
  }

  function isPrivileged(title: string): boolean {
    const t = title.toLowerCase();
    if (!t) return false;
    return (
      t === "gm" ||
      t.startsWith("gm ") ||
      t.startsWith("gm:") ||
      t.startsWith("dm ") ||
      t.includes("secret") ||
      t.includes("spoiler")
    );
  }

  const split = $derived(buildSections(bodyHtml, headingToHide));

  // Reset collapse state when another document (or version) is loaded.
  $effect(() => {
    void bodyHtml;
    collapsed = Object.fromEntries(
      split.sections
        .filter((s) => s.privileged)
        .map((s) => [s.id, true])
    );
  });

  function toggle(id: number) {
    collapsed[id] = !collapsed[id];
  }
</script>

<div class="scene" class:focus>
  <div class="scene-toolbar">
    <button
      class="focus-btn t-label-lg"
      type="button"
      class:active={focus}
      onclick={() => (focus = !focus)}
      title="Enlarge read-aloud text and dim GM notes"
    >
      {focus ? "Exit Focus" : "Read-Aloud Focus"}
    </button>
  </div>

  <div class="md md-intro" class:focus-dim={focus && !split.introReadAloud}>
    {@html split.intro}
  </div>

  {#each split.sections as s (s.id)}
    <section
      class="md-section"
      class:privileged={s.privileged}
      class:focus-dim={focus && !s.readAloud}
      class:focus-keep={focus && s.readAloud}
    >
      {#if s.title}
        <button
          class="sec-head"
          type="button"
          onclick={() => toggle(s.id)}
          aria-expanded={!collapsed[s.id]}
        >
          <span class="twist t-mono-sm">{collapsed[s.id] ? "▸" : "▾"}</span>
          <h2 class="sec-title t-headline-sm" class:sub={s.level > 2}>{s.title}</h2>
          {#if s.privileged}
            <span class="gm-tag t-mono-sm">GM</span>
          {/if}
        </button>
      {/if}
      {#if !collapsed[s.id]}
        <div class="sec-body md">
          {@html s.html}
        </div>
      {/if}
    </section>
  {/each}
</div>

<style>
  .scene {
    min-height: 0;
  }

  .scene-toolbar {
    display: flex;
    justify-content: flex-end;
    margin-bottom: var(--gutter-sm);
  }

  .focus-btn {
    background: transparent;
    color: var(--text-tertiary);
    border: 1px solid var(--border-static);
    border-radius: var(--rounded);
    padding: 3px 10px;
    cursor: pointer;
    transition: all 100ms ease;
  }

  .focus-btn:hover {
    color: var(--amber);
    border-color: var(--border-focus);
    box-shadow: var(--glow-amber);
  }

  .focus-btn.active {
    color: var(--surface-canvas);
    background: var(--amber);
    border-color: var(--amber);
  }

  .md-intro {
    margin-bottom: var(--gutter-md);
  }

  .md-section {
    margin-bottom: var(--gutter-md);
  }

  .sec-head {
    display: flex;
    align-items: center;
    gap: var(--gutter-xs);
    width: 100%;
    text-align: left;
    background: transparent;
    border: none;
    padding: 2px 0;
    margin: 0 0 var(--gutter-xs);
    cursor: pointer;
    border-radius: var(--rounded-sm);
    transition: background 90ms ease;
  }

  .sec-head:hover {
    background: var(--surface-highlight);
  }

  .twist {
    color: var(--text-tertiary);
    width: 0.875rem;
    flex: none;
  }

  .sec-title {
    margin: 0;
    color: var(--text-primary);
  }

  .sec-title.sub {
    font-family: var(--font-body);
    font-size: var(--fs-body-lg);
    font-weight: 600;
    color: var(--text-secondary);
  }

  .gm-tag {
    color: var(--amber);
    border: 1px solid var(--border-focus);
    background: rgba(224, 169, 83, 0.08);
    border-radius: var(--rounded-sm);
    padding: 0 5px;
    flex: none;
  }

  /* Privileged sections get an amber-tinted edge. */
  .md-section.privileged .sec-body {
    border-left: 2px solid var(--border-focus);
    padding-left: var(--gutter-md);
  }

  /* Focus mode: read-aloud blocks grow, everything else recedes. */
  .scene.focus .md-intro :global(blockquote),
  .scene.focus .sec-body :global(blockquote) {
    font-size: 19px;
    line-height: 1.75;
    padding: var(--gutter-lg) var(--gutter-xl);
    border-left-color: var(--amber-bright);
    box-shadow: var(--glow-amber);
  }

  .scene.focus .focus-dim {
    opacity: 0.25;
    transition: opacity 200ms ease;
  }

  .scene.focus .focus-keep {
    opacity: 1;
  }
</style>
