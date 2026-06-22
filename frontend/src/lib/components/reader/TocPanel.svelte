<script lang="ts">
  import type { TocItem } from "$lib/reader/types";

  interface Props {
    items: TocItem[];
    currentChapter: string;
    onSelect: (href: string) => void;
    onClose: () => void;
  }

  let { items, currentChapter, onSelect, onClose }: Props = $props();

  function chapterBasename(href: string): string {
    const parts = href.split("#");
    return parts[0] ?? href;
  }

  let chapters = $derived(
    items.filter((item) => item.subitems.length === 0).length +
    items.reduce((sum, item) => sum + item.subitems.length, 0)
  );
</script>

<div class="toc-overlay" onclick={onClose} role="presentation">
  <aside class="toc-panel" onclick={(e) => e.stopPropagation()} role="presentation">
    <div class="toc-header">
      <h3>Contents</h3>
      <button class="toc-close" onclick={onClose} aria-label="Close">&times;</button>
    </div>
    <div class="toc-body">
      {#if items.length === 0}
        <p class="toc-empty">No table of contents available.</p>
      {:else}
        <ul class="toc-list">
          {#each items as item}
            <li>
              <button
                class="toc-link toc-parent"
                class:current={currentChapter && chapterBasename(currentChapter) === chapterBasename(item.href)}
                onclick={() => onSelect(item.href)}
              >
                <span class="toc-label">{item.label}</span>
              </button>
              {#if item.subitems && item.subitems.length > 0}
                <ul class="toc-sublist">
                  {#each item.subitems as sub}
                    <li>
                      <button
                        class="toc-link"
                        class:current={currentChapter && chapterBasename(currentChapter) === chapterBasename(sub.href)}
                        onclick={() => onSelect(sub.href)}
                      >
                        <span class="toc-label">{sub.label}</span>
                      </button>
                    </li>
                  {/each}
                </ul>
              {/if}
            </li>
          {/each}
        </ul>
      {/if}
    </div>
  </aside>
</div>

<style>
  .toc-overlay {
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

  .toc-panel {
    width: 300px;
    max-width: 85vw;
    background: var(--reader-panel-bg, #fff);
    color: var(--reader-panel-fg, #1a1a1a);
    display: flex;
    flex-direction: column;
    box-shadow: 4px 0 24px rgba(0, 0, 0, 0.18);
    animation: slide-in 0.2s ease-out;
  }

  @keyframes slide-in {
    from { transform: translateX(-100%); }
    to   { transform: translateX(0); }
  }

  .toc-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.25rem;
    border-bottom: 1px solid var(--reader-border, rgba(0, 0, 0, 0.08));
    flex-shrink: 0;
  }

  .toc-header h3 {
    margin: 0;
    font-size: 1rem;
    font-weight: 600;
  }

  .toc-close {
    background: none;
    border: none;
    font-size: 1.25rem;
    cursor: pointer;
    color: var(--reader-muted, #9ca3af);
    padding: 0.25rem;
    line-height: 1;
    border-radius: 4px;
  }
  .toc-close:hover {
    color: var(--reader-panel-fg, #1a1a1a);
    background: var(--reader-hover, rgba(0, 0, 0, 0.05));
  }

  .toc-body {
    flex: 1;
    overflow-y: auto;
    padding: 0.5rem 0;
  }

  .toc-empty {
    color: var(--reader-muted, #9ca3af);
    font-size: 0.85rem;
    padding: 1rem 1.25rem;
  }

  .toc-list,
  .toc-sublist {
    list-style: none;
    padding: 0;
    margin: 0;
  }

  .toc-sublist {
    border-left: 1px solid var(--reader-border, rgba(0, 0, 0, 0.08));
    margin-left: 1.5rem;
  }

  .toc-link {
    display: flex;
    align-items: center;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    padding: 0.55rem 1.25rem;
    cursor: pointer;
    color: var(--reader-panel-fg, #1a1a1a);
    font-size: 0.88rem;
    transition: background 0.1s;
    border-radius: 0;
  }
  .toc-link:hover {
    background: var(--reader-hover, rgba(0, 0, 0, 0.04));
  }
  .toc-link.current {
    color: #4f46e5;
    font-weight: 600;
  }

  .toc-parent {
    font-weight: 500;
  }

  .toc-label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
</style>
