<script lang="ts">
  interface Props {
    progress: number;
    minutesLeft: number;
    totalSections: number;
    currentSectionIndex: number;
  }

  let { progress, minutesLeft, totalSections, currentSectionIndex }: Props = $props();

  let timeLabel = $derived(
    minutesLeft > 0
      ? minutesLeft === 1
        ? "~1 min left"
        : `~${minutesLeft} mins left`
      : ""
  );
</script>

<footer class="reader-footer">
  <div class="footer-content">
    <span class="footer-time">{timeLabel}</span>
    {#if totalSections > 0}
      <div class="section-dots" aria-hidden="true">
        {#each { length: Math.min(totalSections, 40) } as _, i}
          <span
            class="section-dot"
            class:active={i <= Math.round((currentSectionIndex / (totalSections - 1)) * (Math.min(totalSections, 40) - 1))}
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
    background: linear-gradient(to top, var(--reader-bg, #fff) 60%, transparent);
    user-select: none;
    -webkit-user-select: none;
  }

  .footer-content {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.75rem;
  }

  .footer-time,
  .footer-percent {
    font-size: 0.7rem;
    color: var(--reader-muted, #9ca3af);
    white-space: nowrap;
    min-width: 72px;
  }

  .footer-percent {
    text-align: right;
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
