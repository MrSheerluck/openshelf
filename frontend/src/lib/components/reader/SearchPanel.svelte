<script lang="ts">
  import type { SearchResult } from "$lib/reader/types";

  interface Props {
    query: string;
    searching: boolean;
    results: SearchResult[];
    onQueryChange: (query: string) => void;
    onSelect: (href: string) => void;
    onClose: () => void;
  }

  let { query, searching, results, onQueryChange, onSelect, onClose }: Props =
    $props();
</script>

<div class="panel-overlay" onclick={onClose} role="presentation">
  <aside class="panel" onclick={(e) => e.stopPropagation()} role="presentation">
    <div class="panel-header">
      <h3>Search</h3>
      <button class="panel-close" onclick={onClose} aria-label="Close"
        >&times;</button
      >
    </div>
    <div class="search-box">
      <input
        type="search"
        placeholder="Search this book"
        value={query}
        oninput={(e) =>
          onQueryChange((e.currentTarget as HTMLInputElement).value)}
      />
    </div>
    <div class="panel-body">
      {#if searching}
        <p class="empty">Searching chapters...</p>
      {:else if query.trim().length < 2}
        <p class="empty">Type at least 2 characters to search.</p>
      {:else if results.length === 0}
        <p class="empty">No matches found.</p>
      {:else}
        {#each results as result (result.id)}
          <button class="result" onclick={() => onSelect(result.href)}>
            <p class="result-chapter">{result.chapterLabel}</p>
            <p class="result-excerpt">{result.excerpt}</p>
          </button>
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
    width: 360px;
    max-width: 88vw;
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

  .panel-close {
    background: none;
    border: none;
    color: var(--reader-muted, #9ca3af);
    cursor: pointer;
    font-size: 1.1rem;
  }

  .search-box {
    padding: 0.9rem 1rem 0.5rem;
  }

  .search-box input {
    width: 100%;
    box-sizing: border-box;
    border-radius: 10px;
    border: 1px solid var(--reader-border, rgba(0, 0, 0, 0.12));
    background: var(--reader-hover, rgba(0, 0, 0, 0.03));
    color: inherit;
    padding: 0.7rem 0.85rem;
    font: inherit;
  }

  .panel-body {
    overflow-y: auto;
    padding: 0.25rem 0 0.75rem;
  }

  .empty {
    padding: 0.75rem 1rem;
    color: var(--reader-muted, #9ca3af);
    font-size: 0.85rem;
  }

  .result {
    width: 100%;
    text-align: left;
    border: none;
    background: none;
    color: inherit;
    cursor: pointer;
    padding: 0.8rem 1rem;
    border-bottom: 1px solid var(--reader-border, rgba(0, 0, 0, 0.04));
  }

  .result-chapter,
  .result-excerpt {
    margin: 0;
  }

  .result-chapter {
    font-size: 0.75rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.03em;
    color: var(--reader-muted, #9ca3af);
    margin-bottom: 0.3rem;
  }

  .result-excerpt {
    font-size: 0.85rem;
    line-height: 1.45;
  }
</style>
