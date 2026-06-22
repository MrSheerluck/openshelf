<script lang="ts">
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import { api } from "$lib/api";
  import { onMount } from "svelte";

  interface Book {
    id: string;
    title: string;
    author: string | null;
    format: string;
  }

  let book = $state<Book | null>(null);
  let loading = $state(true);
  let error = $state("");
  let viewerEl = $state<HTMLDivElement | null>(null);
  let rendition: any = null;
  let fileBlobUrl = $state<string | null>(null);

  const API_URL = import.meta.env.VITE_API_URL ?? "http://localhost:3001";
  const id = $derived(String(page.params.id));

  function fileUrl(): string {
    return `${API_URL}/api/books/${id}/file`;
  }

  async function loadBook() {
    try {
      const res = await api(`/api/books/${id}`);
      if (res.ok) {
        book = await res.json();
      } else {
        error = "Book not found";
      }
    } catch {
      error = "Failed to load book";
    } finally {
      loading = false;
    }
  }

  function renderEpub() {
    if (!viewerEl || !book || !fileBlobUrl) return;
    loadEpub(fileBlobUrl, viewerEl);
  }

  async function loadEpub(url: string, element: HTMLDivElement) {
    const ePub = (await import("epubjs")).default;

    const epub: any = ePub(url);
    rendition = epub.renderTo(element, {
      width: "100%",
      height: "100%",
      flow: "paginated",
      spread: "none",
    });
    rendition.display();
  }

  async function prefetchBookFile() {
    try {
      const res = await fetch(fileUrl(), { credentials: "include" });
      if (!res.ok) {
        error = "Failed to load book file";
        loading = false;
        return;
      }
      const blob = await res.blob();
      if (fileBlobUrl) URL.revokeObjectURL(fileBlobUrl);
      fileBlobUrl = URL.createObjectURL(blob);
      loading = false;
    } catch {
      error = "Failed to load book file";
      loading = false;
    }
  }

  $effect(() => {
    loadBook();
  });

  $effect(() => {
    if (book) {
      prefetchBookFile();
    }
  });

  $effect(() => {
    if (book && viewerEl && fileBlobUrl && book.format === "epub") {
      renderEpub();
      return () => {
        if (rendition) {
          rendition.destroy();
          rendition = null;
        }
      };
    }
  });

  function goBack() {
    goto("/");
  }

  function prevPage() {
    if (rendition) rendition.prev();
  }

  function nextPage() {
    if (rendition) rendition.next();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (book?.format === "epub") {
      if (e.key === "ArrowLeft") prevPage();
      if (e.key === "ArrowRight") nextPage();
    }
  }
</script>

<svelte:head>
  <title>{book?.title ?? "Reader"} - OpenShelf</title>
</svelte:head>

<svelte:window onkeydown={handleKeydown} />

<div class="reader-app">
  <header class="reader-header">
    <button class="back-btn" onclick={goBack}>&larr; Library</button>
    <span class="reader-title">{book?.title ?? "Loading..."}</span>
    <span class="reader-spacer"></span>
  </header>

  <main class="reader-main">
    {#if loading}
      <p class="reader-status">Loading...</p>
    {:else if error}
      <p class="reader-status error">{error}</p>
    {:else if !fileBlobUrl}
      <p class="reader-status">Loading book...</p>
    {:else if book?.format === "epub"}
      <div class="epub-reader" bind:this={viewerEl}></div>
      <div class="epub-controls">
        <button class="nav-btn" onclick={prevPage}>&lsaquo;</button>
        <button class="nav-btn" onclick={nextPage}>&rsaquo;</button>
      </div>
    {:else if book?.format === "pdf"}
      <embed src={fileBlobUrl} type="application/pdf" class="pdf-viewer" />
    {:else if book?.format === "mobi"}
      <div class="unsupported-format">
        <p>MOBI files cannot be displayed in the browser.</p>
        <a href={fileBlobUrl} download class="download-link">Download file</a>
      </div>
    {/if}
  </main>
</div>

<style>
  .reader-app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    font-family: system-ui, sans-serif;
    background: #f5f5f5;
  }

  .reader-header {
    display: flex;
    align-items: center;
    gap: 1rem;
    padding: 0.5rem 1rem;
    background: #fff;
    border-bottom: 1px solid #e5e7eb;
    flex-shrink: 0;
  }

  .back-btn {
    background: none;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    padding: 0.35rem 0.75rem;
    cursor: pointer;
    font-size: 0.85rem;
    color: #555;
    white-space: nowrap;
  }
  .back-btn:hover {
    background: #f5f5f5;
  }

  .reader-title {
    font-size: 0.9rem;
    color: #333;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .reader-spacer {
    flex: 1;
  }

  .reader-main {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    overflow: hidden;
    position: relative;
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
  }

  .epub-controls {
    position: fixed;
    bottom: 1rem;
    left: 50%;
    transform: translateX(-50%);
    display: flex;
    gap: 0.5rem;
    z-index: 10;
  }

  .nav-btn {
    width: 40px;
    height: 40px;
    border-radius: 50%;
    border: 1px solid #d1d5db;
    background: rgba(255, 255, 255, 0.9);
    font-size: 1.25rem;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    color: #555;
    backdrop-filter: blur(8px);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  }
  .nav-btn:hover {
    background: #fff;
    color: #111;
  }

  .pdf-viewer {
    width: 100%;
    height: 100%;
    border: none;
  }

  .unsupported-format {
    text-align: center;
    color: #888;
  }

  .unsupported-format p {
    margin-bottom: 0.75rem;
  }

  .download-link {
    display: inline-block;
    padding: 0.5rem 1rem;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    text-decoration: none;
    color: #111;
    font-size: 0.9rem;
  }
  .download-link:hover {
    background: #f5f5f5;
  }
</style>
