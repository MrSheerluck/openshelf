<script lang="ts">
  import { highlightColors } from "$lib/reader/highlights.svelte";
  import type { Highlight, HighlightColor } from "$lib/reader/types";

  interface Props {
    highlight: Highlight;
    x: number;
    y: number;
    onSave: (note: string | null) => void;
    onSetColor: (color: HighlightColor) => void;
    onDelete: () => void;
    onClose: () => void;
  }

  let { highlight, x, y, onSave, onSetColor, onDelete, onClose }: Props = $props();

  let note = $state("");

  $effect(() => {
    note = highlight.note ?? "";
  });

  let style = $derived(() => {
    const w = 320;
    const left = Math.min(x, window.innerWidth - w - 16);
    const top = Math.min(y + 8, window.innerHeight - 200);
    return `left: ${Math.max(8, left)}px; top: ${top}px; width: ${w}px;`;
  });

  function save() {
    onSave(note.trim() || null);
  }
</script>

<div
  class="note-popup"
  style={style()}
  onclick={(e) => e.stopPropagation()}
  onkeydown={(e) => e.stopPropagation()}
  role="dialog"
  tabindex="-1"
>
  <div class="note-header">
    <span class="note-quote">"{highlight.text.slice(0, 80)}{highlight.text.length > 80 ? '...' : ''}"</span>
    <button class="note-close" onclick={onClose} aria-label="Close">&times;</button>
  </div>
  <textarea
    class="note-input"
    placeholder="Add a note..."
    bind:value={note}
    rows="3"
  ></textarea>
  <div class="note-colors" role="toolbar" aria-label="Highlight colors">
    {#each highlightColors as color}
      <button
        class="color-btn"
        class:active={highlight.color === color.value}
        style="background: {color.css};"
        onclick={() => onSetColor(color.value)}
        aria-label={`Set ${color.label} highlight`}
      ></button>
    {/each}
  </div>
  <div class="note-actions">
    <button class="note-delete" onclick={onDelete}>Delete highlight</button>
    <button class="note-save" onclick={save}>Save note</button>
  </div>
</div>

<style>
  .note-popup {
    position: fixed;
    z-index: 71;
    background: var(--reader-panel-bg, #fff);
    color: var(--reader-panel-fg, #1a1a1a);
    border: 1px solid var(--reader-border, rgba(0, 0, 0, 0.15));
    border-radius: 12px;
    box-shadow: 0 6px 24px rgba(0, 0, 0, 0.2);
    padding: 0.6rem 0.75rem 0.75rem;
    animation: pop 0.15s ease-out;
  }

  @keyframes pop {
    from { opacity: 0; transform: scale(0.95); }
    to   { opacity: 1; transform: scale(1); }
  }

  .note-header {
    display: flex;
    align-items: flex-start;
    gap: 0.5rem;
    margin-bottom: 0.4rem;
  }

  .note-quote {
    font-size: 0.8rem;
    color: var(--reader-muted, #9ca3af);
    font-style: italic;
    line-height: 1.4;
    flex: 1;
  }

  .note-close {
    background: none;
    border: none;
    font-size: 1.1rem;
    cursor: pointer;
    color: var(--reader-muted, #9ca3af);
    padding: 0;
    line-height: 1;
    flex-shrink: 0;
  }
  .note-close:hover {
    color: var(--reader-panel-fg, #1a1a1a);
  }

  .note-input {
    width: 100%;
    background: var(--reader-hover, rgba(0, 0, 0, 0.04));
    border: 1px solid var(--reader-border, rgba(0, 0, 0, 0.1));
    border-radius: 8px;
    padding: 0.5rem;
    font-size: 0.85rem;
    color: inherit;
    font-family: inherit;
    resize: none;
    box-sizing: border-box;
  }
  .note-input:focus {
    outline: none;
    border-color: #4f46e5;
  }

  .note-actions {
    display: flex;
    justify-content: space-between;
    margin-top: 0.5rem;
    gap: 0.4rem;
  }

  .note-colors {
    display: flex;
    gap: 0.35rem;
    margin-top: 0.65rem;
  }

  .color-btn {
    width: 22px;
    height: 22px;
    border-radius: 999px;
    border: 2px solid transparent;
    cursor: pointer;
    padding: 0;
    box-shadow: inset 0 0 0 1px rgba(0, 0, 0, 0.08);
  }

  .color-btn.active {
    border-color: var(--reader-panel-fg, #1a1a1a);
  }

  .note-delete {
    background: none;
    border: none;
    color: #dc2626;
    font-size: 0.8rem;
    cursor: pointer;
    padding: 0.3rem 0.4rem;
    border-radius: 6px;
  }
  .note-delete:hover {
    background: rgba(220, 38, 38, 0.08);
  }

  .note-save {
    background: #4f46e5;
    color: #fff;
    border: none;
    font-size: 0.8rem;
    cursor: pointer;
    padding: 0.3rem 0.7rem;
    border-radius: 6px;
  }
  .note-save:hover {
    background: #4338ca;
  }
</style>
