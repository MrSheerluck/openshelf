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

    if (elapsed > TAP_MAX_MS || Math.abs(dx) > TAP_THRESHOLD) {
      return;
    }

    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
    const relativeX = e.clientX - rect.left;
    const zone = relativeX / rect.width;

    if (zone < LEFT_ZONE_RATIO) {
      onPrev();
    } else {
      onNext();
    }
  }
</script>

<div
  class="reader-main"
  class:page-turning={pageTurning !== null}
  class:turning-forward={pageTurning === "forward"}
  class:turning-back={pageTurning === "backward"}
  role="presentation"
  onpointerdown={book?.format === "epub" ? handlePointerDown : undefined}
  onpointermove={book?.format === "epub" ? handlePointerMove : undefined}
  onpointerup={book?.format === "epub" ? handlePointerUp : undefined}
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
  {/if}
</div>

<style>
  .reader-main {
    flex: 1;
    position: relative;
    overflow: hidden;
    min-height: 0;
    touch-action: none;
    -webkit-user-select: none;
    user-select: none;
  }

  .reader-status {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #888;
    font-size: 0.95rem;
  }
  .reader-status.error {
    color: #dc2626;
  }

  .epub-reader {
    position: absolute;
    inset: 0;
    overflow: hidden;
    transition: opacity 0.12s ease-out;
  }
  .reader-main.turning-forward .epub-reader,
  .reader-main.turning-back .epub-reader {
    opacity: 0.88;
  }
  .epub-reader :global(iframe) {
    border: none;
    width: 100%;
    height: 100%;
    background: transparent;
    pointer-events: none !important;
  }

  .pdf-viewer {
    position: absolute;
    inset: 0;
    border: none;
    display: block;
  }

  .unsupported-format {
    position: absolute;
    inset: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
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
