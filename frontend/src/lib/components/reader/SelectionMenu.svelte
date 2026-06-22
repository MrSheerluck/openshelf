<script lang="ts">
  import type { HighlightColor } from "$lib/reader/types";
  import { highlightColors } from "$lib/reader/highlights.svelte";

  interface Props {
    x: number;
    y: number;
    text: string;
    onHighlight: (color: HighlightColor) => void;
    onDictionary: () => void;
    onCopy: () => void;
    onClose: () => void;
  }

  let { x, y, text, onHighlight, onDictionary, onCopy, onClose }: Props = $props();

  let style = $derived(
    `left: ${Math.min(x, window.innerWidth - 280)}px; top: ${Math.max(y - 56, 8)}px;`
  );

  function handleCopy() {
    navigator.clipboard.writeText(text).catch(() => {});
    onCopy();
  }
</script>

<div
  class="selection-menu"
  style={style}
  onclick={(e) => e.stopPropagation()}
  onkeydown={(e) => e.stopPropagation()}
  role="toolbar"
  tabindex="-1"
>
  <div class="color-buttons">
    {#each highlightColors as c}
      <button
        class="color-btn"
        style="background: {c.css};"
        onclick={() => onHighlight(c.value)}
        title="Highlight {c.label}"
        aria-label="Highlight {c.label}"
      ></button>
    {/each}
  </div>
  <div class="divider"></div>
  <button class="action-btn" onclick={() => onDictionary()} aria-label="Look up">
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <path d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"></path>
      <path d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"></path>
    </svg>
  </button>
  <button class="action-btn" onclick={handleCopy} aria-label="Copy">
    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect>
      <path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path>
    </svg>
  </button>
</div>

<style>
  .selection-menu {
    position: fixed;
    z-index: 70;
    display: flex;
    align-items: center;
    gap: 0.25rem;
    background: var(--reader-panel-bg, #fff);
    color: var(--reader-panel-fg, #1a1a1a);
    border: 1px solid var(--reader-border, rgba(0, 0, 0, 0.15));
    border-radius: 10px;
    padding: 0.3rem 0.4rem;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.18);
    animation: pop 0.12s ease-out;
  }

  @keyframes pop {
    from { opacity: 0; transform: scale(0.92); }
    to   { opacity: 1; transform: scale(1); }
  }

  .color-buttons {
    display: flex;
    gap: 0.2rem;
  }

  .color-btn {
    width: 24px;
    height: 24px;
    border-radius: 50%;
    border: 2px solid var(--reader-border, rgba(0, 0, 0, 0.1));
    cursor: pointer;
    padding: 0;
    transition: transform 0.1s;
  }
  .color-btn:hover {
    transform: scale(1.15);
  }

  .divider {
    width: 1px;
    height: 20px;
    background: var(--reader-border, rgba(0, 0, 0, 0.1));
    margin: 0 0.15rem;
  }

  .action-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 30px;
    height: 30px;
    border: none;
    border-radius: 6px;
    background: none;
    cursor: pointer;
    color: var(--reader-panel-fg, #1a1a1a);
    padding: 0;
    transition: background 0.1s;
  }
  .action-btn:hover {
    background: var(--reader-hover, rgba(0, 0, 0, 0.06));
  }
</style>
