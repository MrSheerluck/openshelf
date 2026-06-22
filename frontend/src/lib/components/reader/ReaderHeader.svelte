<script lang="ts">
  interface Props {
    title: string;
    showTocButton: boolean;
    isBookmarked: boolean;
    onBack: () => void;
    onToggleSearch: () => void;
    onToggleBookmarks: () => void;
    onToggleBookmark: () => void;
    onToggleToc: () => void;
    onToggleTypography: () => void;
    onToggleHighlights: () => void;
  }

  let {
    title,
    showTocButton,
    isBookmarked,
    onBack,
    onToggleSearch,
    onToggleBookmarks,
    onToggleBookmark,
    onToggleToc,
    onToggleTypography,
    onToggleHighlights,
  }: Props = $props();
</script>

<header class="reader-header">
  <button class="header-btn" onclick={onBack} title="Back to library" aria-label="Back">
    <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <polyline points="15 18 9 12 15 6"></polyline>
    </svg>
  </button>
  <span class="reader-title">{title}</span>
  <div class="header-actions">
    {#if showTocButton}
      <button class="header-btn" onclick={onToggleSearch} title="Search in book" aria-label="Search in book">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="11" cy="11" r="8"></circle>
          <path d="m21 21-4.3-4.3"></path>
        </svg>
      </button>
      <button class="header-btn" onclick={onToggleToc} title="Table of contents" aria-label="Contents">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <line x1="8" y1="6" x2="21" y2="6"></line>
          <line x1="8" y1="12" x2="21" y2="12"></line>
          <line x1="8" y1="18" x2="21" y2="18"></line>
          <line x1="3" y1="6" x2="3.01" y2="6"></line>
          <line x1="3" y1="12" x2="3.01" y2="12"></line>
          <line x1="3" y1="18" x2="3.01" y2="18"></line>
        </svg>
      </button>
      <button class="header-btn" onclick={onToggleBookmarks} title="Bookmarks" aria-label="Bookmarks">
        <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M19 21 12 16 5 21V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"></path>
        </svg>
      </button>
      <button
        class="header-btn"
        class:active={isBookmarked}
        onclick={onToggleBookmark}
        title={isBookmarked ? "Remove bookmark" : "Add bookmark"}
        aria-label={isBookmarked ? "Remove bookmark" : "Add bookmark"}
      >
        <svg width="18" height="18" viewBox="0 0 24 24" fill={isBookmarked ? "currentColor" : "none"} stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M19 21 12 16 5 21V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2z"></path>
        </svg>
      </button>
    {/if}
    <button
      class="header-btn"
      onclick={onToggleHighlights}
      title="Highlights"
      aria-label="Highlights"
    >
      <svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="m9 11-6 6v3h9l3-3"></path>
        <path d="m22 12-4.6 4.6a2 2 0 0 1-2.8 0l-5.2-5.2a2 2 0 0 1 0-2.8L14 4"></path>
      </svg>
    </button>
    <button
      class="header-btn typo-btn"
      onclick={onToggleTypography}
      title="Typography &amp; themes"
      aria-label="Typography settings"
    >Aa</button>
  </div>
</header>

<style>
  .reader-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.5rem 0.75rem;
    background: var(--reader-bg, #fff);
    color: var(--reader-fg, #1a1a1a);
    border-bottom: 1px solid var(--reader-border, rgba(0, 0, 0, 0.08));
    flex-shrink: 0;
    user-select: none;
    -webkit-user-select: none;
  }

  .header-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    background: none;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    color: inherit;
    padding: 0;
    flex-shrink: 0;
    transition: background 0.15s;
  }
  .header-btn:hover {
    background: var(--reader-hover, rgba(0, 0, 0, 0.06));
  }
  .header-btn:active {
    background: var(--reader-hover, rgba(0, 0, 0, 0.1));
  }

  .header-btn.active {
    color: #2563eb;
  }

  .typo-btn {
    font-size: 0.9rem;
    font-weight: 600;
    letter-spacing: 0.02em;
  }

  .reader-title {
    font-size: 0.85rem;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 40vw;
    opacity: 0.8;
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }
</style>
