<script lang="ts">
  import type { Book } from "$lib/reader/types";
  import { buildReaderCss } from "$lib/reader/epub.svelte";
  import { defaultTypography, themes } from "$lib/reader/themes";

  const API_URL = import.meta.env.VITE_API_URL ?? "http://localhost:3001";

  interface Props {
    book: Book | null;
    fileBlobUrl: string | null;
    loading: boolean;
    error: string;
    pageTurning: "forward" | "backward" | null;
    firstChapterHtml: string | null;
    firstChapterHref: string | null;
    onViewportEl: (el: HTMLDivElement) => void;
    onDownload: () => void;
  }

  let {
    book,
    fileBlobUrl,
    loading,
    error,
    pageTurning,
    firstChapterHtml,
    firstChapterHref,
    onViewportEl,
    onDownload,
  }: Props = $props();

  let epubReaderEl = $state<HTMLDivElement | null>(null);

  $effect(() => {
    if (epubReaderEl) onViewportEl(epubReaderEl);
  });

  function previewSrcdoc(): string {
    const css = buildReaderCss("light", defaultTypography);
    const dir = firstChapterHref
      ? firstChapterHref.split("/").slice(0, -1).join("/") + "/"
      : "";
    const baseHref = `/api/books/${book?.id}/resource/${dir}`;
    return `<!DOCTYPE html>
<html>
<head>
  <meta charset="utf-8">
  <base href="${baseHref}">
  <style>${css}</style>
</head>
<body>${firstChapterHtml ?? ""}</body>
</html>`;
  }

  function coverUrl(): string | null {
    return book ? `${API_URL}/api/books/${book.id}/cover` : null;
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
    <div class="loading-cover">
      {#if book?.id}
        <img src={coverUrl()!} alt="" class="loading-cover-img" />
      {/if}
      <div class="loading-cover-overlay">
        <div class="loading-spinner"></div>
        <span class="loading-text">Opening your book…</span>
      </div>
    </div>
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
    {#if firstChapterHtml}
      <iframe
        class="preview-frame"
        title="Preview"
        srcdoc={previewSrcdoc()}
        sandbox="allow-same-origin"
      />
    {/if}
    <div bind:this={epubReaderEl} class="epub-reader"></div>
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

  .loading-cover {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    background: #1a1a1a;
  }
  .loading-cover-img {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    opacity: 0.3;
    filter: blur(8px);
  }
  .loading-cover-overlay {
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
  }
  .loading-spinner {
    width: 28px;
    height: 28px;
    border: 3px solid rgba(255,255,255,0.2);
    border-top-color: #fff;
    border-radius: 50%;
    animation: spin 0.7s linear infinite;
  }
  @keyframes spin { to { transform: rotate(360deg); } }
  .loading-text {
    color: rgba(255,255,255,0.7);
    font-size: 0.9rem;
  }

  .epub-reader {
    position: absolute;
    inset: 0;
    overflow: hidden;
    z-index: 1;
    transition: opacity 0.12s ease-out;
  }
  .preview-frame {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    border: none;
    z-index: 3;
    background: transparent;
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