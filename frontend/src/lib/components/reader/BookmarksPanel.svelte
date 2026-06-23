<script lang="ts">
  import type { Bookmark } from "$lib/reader/types";

  interface Props {
    bookmarks: Bookmark[];
    onSelect: (cfi: string) => void;
    onDelete: (id: string) => void;
    onClose: () => void;
  }

  let { bookmarks, onSelect, onDelete, onClose }: Props = $props();
</script>

<div class="panel-overlay" onclick={onClose} role="presentation">
  <aside class="panel" onclick={(e) => e.stopPropagation()} role="presentation">
    <div class="panel-header">
      <h3>Bookmarks</h3>
      <button class="panel-close" onclick={onClose} aria-label="Close"
        >&times;</button
      >
    </div>
    <div class="panel-body">
      {#if bookmarks.length === 0}
        <p class="empty">
          No bookmarks yet. Use the flag button to save your place.
        </p>
      {:else}
        {#each bookmarks as bookmark (bookmark.id)}
          <div
            class="item"
            role="button"
            tabindex="0"
            onclick={() => onSelect(bookmark.cfi)}
            onkeydown={(e) => e.key === "Enter" && onSelect(bookmark.cfi)}
          >
            <div class="item-copy">
              <p class="item-title">
                {bookmark.chapterLabel ??
                  `Chapter ${bookmark.chapterIndex + 1}`}
              </p>
              <p class="item-meta">Saved spot</p>
            </div>
            <button
              class="item-delete"
              onclick={(e) => {
                e.stopPropagation();
                onDelete(bookmark.id);
              }}
              aria-label="Delete bookmark">&times;</button
            >
          </div>
        {/each}
      {/if}
    </div>
  </aside>
</div>

<style>
  .panel-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    justify-content: flex-end;
    z-index: 50;
  }

  .panel {
    width: 320px;
    max-width: 85vw;
    background: var(--reader-panel-bg, #fff);
    color: var(--reader-panel-fg, #1a1a1a);
    box-shadow: -4px 0 24px rgba(0, 0, 0, 0.18);
    display: flex;
    flex-direction: column;
  }

  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--reader-border, rgba(0, 0, 0, 0.08));
  }

  .panel-header h3 {
    margin: 0;
    font-size: 1rem;
  }

  .panel-close,
  .item-delete {
    background: none;
    border: none;
    color: var(--reader-muted, #9ca3af);
    cursor: pointer;
    font-size: 1.1rem;
  }

  .panel-body {
    overflow-y: auto;
    padding: 0.5rem 0;
  }

  .empty {
    padding: 1rem 1.25rem;
    color: var(--reader-muted, #9ca3af);
    font-size: 0.85rem;
  }

  .item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.8rem 1rem;
    background: none;
    border: none;
    border-bottom: 1px solid var(--reader-border, rgba(0, 0, 0, 0.04));
    text-align: left;
    cursor: pointer;
  }

  .item-copy {
    flex: 1;
    min-width: 0;
  }

  .item-title,
  .item-meta {
    margin: 0;
  }

  .item-title {
    font-size: 0.86rem;
    font-weight: 600;
  }

  .item-meta {
    font-size: 0.74rem;
    color: var(--reader-muted, #9ca3af);
    margin-top: 0.2rem;
  }
</style>
