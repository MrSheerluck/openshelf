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
  }

  let {
    book,
    fileBlobUrl,
    loading,
    error,
    pageTurning,
    onViewerMount,
    onDownload,
  }: Props = $props();

  let viewerEl = $state<HTMLDivElement | null>(null);

  $effect(() => {
    if (book && book.format === "epub" && viewerEl && !loading && !error) {
      onViewerMount(viewerEl);
    }
  });
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
  {/if}
</div>

<style>
  .reader-main {
    flex: 1;
    position: relative;
    overflow: hidden;
    min-height: 0;
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
  }

  .pdf-viewer {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
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
