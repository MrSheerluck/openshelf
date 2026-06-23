<script lang="ts">
  interface Props {
    progress: number;
    totalSections: number;
    currentSectionIndex: number;
    canJumpBookmarks: boolean;
    onPrevBookmark: () => void;
    onNextBookmark: () => void;
  }

  let {
    progress,
    totalSections,
    currentSectionIndex,
    canJumpBookmarks,
    onPrevBookmark,
    onNextBookmark,
  }: Props = $props();
</script>

<footer class="reader-footer">
  <div class="footer-content">
    {#if canJumpBookmarks}
      <div class="bookmark-jumps">
        <button
          class="jump-btn"
          onclick={onPrevBookmark}
          aria-label="Previous bookmark">Prev bookmark</button
        >
        <button
          class="jump-btn"
          onclick={onNextBookmark}
          aria-label="Next bookmark">Next bookmark</button
        >
      </div>
    {/if}
    {#if totalSections > 0}
      <div class="section-dots" aria-hidden="true">
        {#each { length: Math.min(totalSections, 40) } as _, i}
          <span
            class="section-dot"
            class:active={i <=
              Math.round(
                (currentSectionIndex / (totalSections - 1)) *
                  (Math.min(totalSections, 40) - 1),
              )}
          ></span>
        {/each}
      </div>
    {/if}
    <span class="footer-percent">{progress}%</span>
  </div>
</footer>

<style>
  .reader-footer {
    padding: 0.4rem 1rem 0.5rem;
    background: linear-gradient(
      to top,
      var(--reader-bg, #fff) 60%,
      transparent
    );
    user-select: none;
    -webkit-user-select: none;
  }

  .footer-content {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 0.75rem;
  }

  .bookmark-jumps {
    display: flex;
    gap: 0.35rem;
    margin-right: auto;
  }

  .jump-btn {
    border: 1px solid var(--reader-border, rgba(0, 0, 0, 0.12));
    background: var(--reader-panel-bg, #fff);
    color: var(--reader-panel-fg, #1a1a1a);
    border-radius: 999px;
    font-size: 0.68rem;
    padding: 0.28rem 0.55rem;
    cursor: pointer;
  }

  .footer-percent {
    font-size: 0.7rem;
    color: var(--reader-muted, #9ca3af);
    white-space: nowrap;
  }

  .section-dots {
    display: flex;
    align-items: center;
    gap: 2px;
    flex: 1;
    justify-content: center;
    overflow: hidden;
  }

  .section-dot {
    width: 3px;
    height: 3px;
    border-radius: 50%;
    background: var(--reader-border, rgba(0, 0, 0, 0.12));
    flex-shrink: 0;
    transition: background 0.3s;
  }

  .section-dot.active {
    background: var(--reader-fg, #1a1a1a);
    opacity: 0.35;
  }
</style>
