<script lang="ts">
  import * as auth from "$lib/auth.svelte.ts";
  import { api } from "$lib/api";

  interface Book {
    id: string;
    title: string;
    author: string | null;
    format: string;
    file_size: number | null;
    created_at: string;
  }

  let books = $state<Book[]>([]);
  let uploading = $state(false);
  let dragOver = $state(false);
  let error = $state("");
  let loaded = $state(false);

  async function loadBooks() {
    try {
      const res = await api("/api/books");
      if (res.ok) {
        books = await res.json();
      }
    } catch {
      // ignore
    } finally {
      loaded = true;
    }
  }

  $effect(() => {
    loadBooks();
  });

  async function uploadFile(file: File) {
    uploading = true;
    error = "";
    try {
      const form = new FormData();
      form.append("file", file);

      const res = await api("/api/books", {
        method: "POST",
        body: form,
      });

      if (res.ok) {
        await loadBooks();
      } else {
        error = "Upload failed";
      }
    } catch {
      error = "Upload failed";
    } finally {
      uploading = false;
    }
  }

  function handleFileInput(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (file) uploadFile(file);
    input.value = "";
  }

  function handleDrop(e: DragEvent) {
    e.preventDefault();
    dragOver = false;
    const file = e.dataTransfer?.files?.[0];
    if (file) uploadFile(file);
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    dragOver = true;
  }

  function handleDragLeave() {
    dragOver = false;
  }

  function formatSize(bytes: number | null): string {
    if (!bytes) return "";
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }
</script>

<svelte:head>
  <title>Library - OpenShelf</title>
</svelte:head>

<div class="app">
  <aside class="sidebar">
    <h1>OpenShelf</h1>
    <nav>
      <a href="/" class="active">Library</a>
    </nav>
    <div class="spacer"></div>
    <button onclick={() => auth.logout()}>Sign out</button>
  </aside>

  <main class="content">
    {#if !loaded}
      <p class="status">Loading...</p>
    {:else if books.length === 0}
      <div class="empty-state">
        <h2>Upload your first book</h2>
        <p>Drag and drop an EPUB file or click to browse.</p>
        <label class="upload-area" class:dragover={dragOver}>
          <input
            type="file"
            accept=".epub"
            onchange={handleFileInput}
            disabled={uploading}
          />
          {#if uploading}
            <span>Uploading...</span>
          {:else}
            <span>Choose file or drag here</span>
          {/if}
        </label>
        {#if error}
          <p class="error">{error}</p>
        {/if}
      </div>
    {:else}
      <div class="library">
        <label class="upload-btn">
          <input
            type="file"
            accept=".epub"
            onchange={handleFileInput}
            disabled={uploading}
          />
          {uploading ? "Uploading..." : "+ Add book"}
        </label>
        {#if error}
          <p class="error">{error}</p>
        {/if}
        <div class="book-grid">
          {#each books as book}
            <div class="book-card">
              <div class="book-cover" class:epub={book.format === "epub"} class:pdf={book.format === "pdf"}>
                <span class="cover-format">{book.format.toUpperCase()}</span>
              </div>
              <div class="book-info">
                <p class="book-title">{book.title}</p>
                {#if book.author}
                  <p class="book-author">{book.author}</p>
                {/if}
                <p class="book-meta">{formatSize(book.file_size)}</p>
              </div>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </main>
</div>

<style>
  .app {
    display: flex;
    height: 100vh;
    font-family: system-ui, sans-serif;
  }
  .sidebar {
    width: 240px;
    background: #f9fafb;
    border-right: 1px solid #e5e7eb;
    display: flex;
    flex-direction: column;
    padding: 1.5rem 1rem;
  }
  .sidebar h1 {
    font-size: 1.25rem;
    margin: 0 0 2rem 0;
  }
  nav {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }
  nav a {
    text-decoration: none;
    color: #111;
    padding: 0.4rem 0.6rem;
    border-radius: 6px;
    font-size: 0.9rem;
  }
  nav a.active {
    background: #e5e7eb;
    font-weight: 500;
  }
  .spacer {
    flex: 1;
  }
  .sidebar button {
    background: none;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    padding: 0.4rem;
    cursor: pointer;
    font-size: 0.85rem;
    color: #555;
  }
  .content {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .status {
    color: #888;
    font-size: 0.95rem;
  }
  .empty-state {
    text-align: center;
    color: #888;
  }
  .empty-state h2 {
    color: #333;
    margin-bottom: 0.5rem;
    font-size: 1.25rem;
  }
  .empty-state p {
    margin-bottom: 1.25rem;
  }
  .upload-area {
    display: inline-block;
    border: 2px dashed #d1d5db;
    border-radius: 10px;
    padding: 2rem 3rem;
    cursor: pointer;
    color: #666;
    font-size: 0.95rem;
    transition: border-color 0.2s, background 0.2s;
  }
  .upload-area:hover,
  .upload-area.dragover {
    border-color: #111;
    background: #f5f5f5;
    color: #111;
  }
  .upload-area input {
    display: none;
  }
  .error {
    color: #dc2626;
    font-size: 0.85rem;
    margin-top: 0.75rem;
  }
  .library {
    width: 100%;
    height: 100%;
    padding: 1.5rem;
    overflow-y: auto;
  }
  .upload-btn {
    display: inline-block;
    padding: 0.5rem 1rem;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.9rem;
    color: #111;
    margin-bottom: 1rem;
    transition: background 0.15s;
  }
  .upload-btn:hover {
    background: #f5f5f5;
  }
  .upload-btn input {
    display: none;
  }
  .book-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: 1rem;
  }
  .book-card {
    cursor: pointer;
    border-radius: 8px;
    overflow: hidden;
    transition: transform 0.15s, box-shadow 0.15s;
  }
  .book-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }
  .book-cover {
    aspect-ratio: 2/3;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 6px;
  }
  .book-cover.epub {
    background: linear-gradient(135deg, #e0c3fc, #8ec5fc);
  }
  .book-cover.pdf {
    background: linear-gradient(135deg, #fcc3c3, #fc8e8e);
  }
  .cover-format {
    font-size: 0.75rem;
    font-weight: 600;
    color: rgba(255, 255, 255, 0.8);
    background: rgba(0, 0, 0, 0.15);
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
  }
  .book-info {
    padding: 0.5rem 0.25rem;
  }
  .book-title {
    font-size: 0.85rem;
    font-weight: 500;
    color: #111;
    margin: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .book-author {
    font-size: 0.8rem;
    color: #888;
    margin: 0.15rem 0 0;
  }
  .book-meta {
    font-size: 0.75rem;
    color: #aaa;
    margin: 0.25rem 0 0;
  }
</style>
