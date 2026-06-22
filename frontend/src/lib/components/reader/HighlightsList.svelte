<script lang="ts">
  import type { Highlight } from "$lib/reader/types";
  import { highlightColors } from "$lib/reader/highlights.svelte";

  interface Props {
    highlights: Highlight[];
    onSelect: (cfiRange: string) => void;
    onDelete: (id: string) => void;
    onClose: () => void;
  }

  let { highlights, onSelect, onDelete, onClose }: Props = $props();

  function colorCss(c: string): string {
    return highlightColors.find((x) => x.value === c)?.css ?? highlightColors[0].css;
  }
</script>

<div class="hl-overlay" onclick={onClose} role="presentation">
  <aside class="hl-panel" onclick={(e) => e.stopPropagation()} role="presentation">
    <div class="hl-header">
      <h3>Highlights</h3>
      <button class="hl-close" onclick={onClose} aria-label="Close">&times;</button>
    </div>
    <div class="hl-body">
      {#if highlights.length === 0}
        <p class="hl-empty">No highlights yet. Select text in the book to create one.</p>
      {:else}
        {#each highlights as h (h.id)}
          <div class="hl-item" role="button" tabindex="0" onclick={() => onSelect(h.cfiRange)} onkeydown={(e) => e.key === "Enter" && onSelect(h.cfiRange)}>
            <div class="hl-bar" style="background: {colorCss(h.color)};"></div>
            <div class="hl-content">
              <p class="hl-text">{h.text}</p>
              {#if h.note}
                <p class="hl-note">{h.note}</p>
              {/if}
            </div>
            <button class="hl-delete" onclick={(e) => { e.stopPropagation(); onDelete(h.id); }} aria-label="Delete highlight">&times;</button>
          </div>
        {/each}
      {/if}
    </div>
  </aside>
</div>

<style>
  .hl-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    z-index: 50;
    animation: fade-in 0.15s ease-out;
  }

  @keyframes fade-in {
    from { opacity: 0; }
    to   { opacity: 1; }
  }

  .hl-panel {
    width: 340px;
    max-width: 85vw;
    margin-left: auto;
    background: var(--reader-panel-bg, #fff);
    color: var(--reader-panel-fg, #1a1a1a);
    display: flex;
    flex-direction: column;
    box-shadow: -4px 0 24px rgba(0, 0, 0, 0.18);
    animation: slide-in 0.2s ease-out;
  }

  @keyframes slide-in {
    from { transform: translateX(100%); }
    to   { transform: translateX(0); }
  }

  .hl-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--reader-border, rgba(0, 0, 0, 0.08));
    flex-shrink: 0;
  }

  .hl-header h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
  }

  .hl-close {
    background: none;
    border: none;
    font-size: 1.25rem;
    cursor: pointer;
    color: var(--reader-muted, #9ca3af);
    padding: 0.25rem;
    line-height: 1;
    border-radius: 4px;
  }
  .hl-close:hover {
    color: var(--reader-panel-fg, #1a1a1a);
    background: var(--reader-hover, rgba(0, 0, 0, 0.05));
  }

  .hl-body {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem 0;
  }

  .hl-empty {
    color: var(--reader-muted, #9ca3af);
    font-size: 0.85rem;
    padding: 1.5rem 1.25rem;
    text-align: center;
  }

  .hl-item {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    padding: 0.6rem 1rem;
    cursor: pointer;
    transition: background 0.1s;
    border-bottom: 1px solid var(--reader-border, rgba(0, 0, 0, 0.04));
  }
  .hl-item:hover {
    background: var(--reader-hover, rgba(0, 0, 0, 0.03));
  }

  .hl-bar {
    width: 3px;
    align-self: stretch;
    border-radius: 2px;
    flex-shrink: 0;
    min-height: 32px;
  }

  .hl-content {
    flex: 1;
    min-width: 0;
  }

  .hl-text {
    font-size: 0.85rem;
    margin: 0 0 0.2rem;
    line-height: 1.4;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .hl-note {
    font-size: 0.78rem;
    color: var(--reader-muted, #9ca3af);
    margin: 0;
    font-style: italic;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    overflow: hidden;
  }

  .hl-delete {
    background: none;
    border: none;
    font-size: 1rem;
    cursor: pointer;
    color: var(--reader-muted, #9ca3af);
    padding: 0;
    line-height: 1;
    flex-shrink: 0;
    opacity: 0;
    transition: opacity 0.1s, color 0.1s;
  }
  .hl-item:hover .hl-delete {
    opacity: 1;
  }
  .hl-delete:hover {
    color: #dc2626;
  }
</style>
