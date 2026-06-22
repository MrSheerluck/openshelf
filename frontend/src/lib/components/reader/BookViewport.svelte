<script lang="ts">
  import type { Book } from "$lib/reader/types";

  interface Props {
    book: Book | null;
    fileBlobUrl: string | null;
    loading: boolean;
    error: string;
    pageTurning: "forward" | "backward" | null;
    onViewerMount: (el: HTMLDivElement) => void;
    onDownload: () => void;
    onPrev: () => void;
    onNext: () => void;
    onToggleControls: () => void;
  }

  let {
    book,
    fileBlobUrl,
    loading,
    error,
    pageTurning,
    onViewerMount,
    onDownload,
    onPrev,
    onNext,
    onToggleControls,
  }: Props = $props();

  let viewerEl = $state<HTMLDivElement | null>(null);

  $effect(() => {
    if (book && book.format === "epub" && viewerEl && !loading && !error) {
      onViewerMount(viewerEl);
    }
  });

  let pointerStartX = $state(0);
  let pointerStartY = $state(0);
  let pointerStartTime = $state(0);
  let pointerMoved = $state(false);
  let pointerActive = $state(false);

  const SWIPE_THRESHOLD = 40;
  const TAP_THRESHOLD = 10;
  const TAP_MAX_MS = 350;
  const LEFT_ZONE_RATIO = 0.3;

  function handlePointerDown(e: PointerEvent) {
    if (!book || book.format !== "epub") return;
    pointerStartX = e.clientX;
    pointerStartY = e.clientY;
    pointerStartTime = Date.now();
    pointerMoved = false;
    pointerActive = true;
  }

  function handlePointerMove(e: PointerEvent) {
    if (!pointerActive) return;
    const dx = Math.abs(e.clientX - pointerStartX);
    const dy = Math.abs(e.clientY - pointerStartY);
    if (dx > 2 || dy > 2) {
      pointerMoved = true;
    }
  }

  function handlePointerUp(e: PointerEvent) {
    if (!pointerActive) return;
    pointerActive = false;
    if (!book || book.format !== "epub") return;

    const dx = e.clientX - pointerStartX;
    const elapsed = Date.now() - pointerStartTime;

    if (Math.abs(dx) > SWIPE_THRESHOLD) {
      if (dx < 0) onNext();
      else onPrev();
      return;
    }

    if (pointerMoved) {
      onToggleControls();
      return;
    }

    if (elapsed < TAP_MAX_MS && Math.abs(dx) < TAP_THRESHOLD) {
      const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
      const relativeX = e.clientX - rect.left;
      const zone = relativeX / rect.width;

      if (zone < LEFT_ZONE_RATIO) {
        onPrev();
      } else {
        onNext();
      }
    } else {
      onToggleControls();
    }
  }

  function handlePointerLeave() {
    pointerActive = false;
  }
</script>

<div
  class="reader-main"
  class:page-turning={pageTurning !== null}
  class:turning-forward={pageTurning === "forward"}
  class:turning-back={pageTurning === "backward"}
  role="presentation"
>
  {#if loading}
    <p class="reader-status">Loading...</p>
  {:else if error}
    <p class="reader-status error">{error}</p>
  {:else if !book}
    <p class="reader-status">Loading...</p>
  {:else if book.format === "pdf"}
    {#if fileBlobUrl}
      <iframe src={fileBlobUrl} title="PDF viewer" class="pdf-viewer"></iframe>
    {:else}
      <p class="reader-status">Loading PDF...</p>
    {/if}
  {:else if book.format === "mobi"}
    <div class="unsupported-format">
      <p>MOBI files cannot be displayed in the browser.</p>
      <button class="download-btn" onclick={onDownload}>Download file</button>
    </div>
  {:else if book.format === "epub"}
    <div class="epub-reader" bind:this={viewerEl}></div>
    <div
      class="touch-layer"
      onpointerdown={handlePointerDown}
      onpointermove={handlePointerMove}
      onpointerup={handlePointerUp}
      onpointerleave={handlePointerLeave}
      onpointercancel={handlePointerLeave}
      role="presentation"
    >
      <div class="touch-zone touch-zone-left"></div>
      <div class="touch-zone touch-zone-right"></div>
    </div>
  {/if}
</div>

<style>
  .reader-main {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    position: relative;
    min-height: 0;
    transition: transform 0.2s ease-out, opacity 0.2s ease-out;
  }

  .reader-main.page-turning {
    transition: transform 0.18s ease-out, opacity 0.18s ease-out;
  }
  .reader-main.turning-forward {
    transform: translateX(-8px);
    opacity: 0.85;
  }
  .reader-main.turning-back {
    transform: translateX(8px);
    opacity: 0.85;
  }

  .reader-status {
    color: #888;
    font-size: 0.95rem;
  }
  .reader-status.error {
    color: #dc2626;
  }

  .epub-reader {
    width: 100%;
    height: 100%;
    overflow: hidden;
    position: relative;
  }
  .epub-reader :global(iframe) {
    border: none;
    width: 100%;
    height: 100%;
    background: transparent;
  }

  .touch-layer {
    position: absolute;
    inset: 0;
    z-index: 3;
    display: flex;
    touch-action: none;
    -webkit-user-select: none;
    user-select: none;
  }

  .touch-zone {
    height: 100%;
  }
  .touch-zone-left {
    width: 30%;
  }
  .touch-zone-right {
    width: 70%;
  }

  .pdf-viewer {
    flex: 1;
    width: 100%;
    height: 100%;
    border: none;
    display: block;
  }

  .unsupported-format {
    text-align: center;
    color: inherit;
  }
  .unsupported-format p {
    margin-bottom: 0.75rem;
  }
  .download-btn {
    display: inline-block;
    padding: 0.5rem 1rem;
    border: 1px solid var(--reader-border, rgba(0, 0, 0, 0.15));
    border-radius: 6px;
    cursor: pointer;
    color: inherit;
    font-size: 0.9rem;
    background: transparent;
  }
  .download-btn:hover {
    background: var(--reader-hover, rgba(0, 0, 0, 0.05));
  }
</style>
