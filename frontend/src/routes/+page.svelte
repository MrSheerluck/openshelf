<script lang="ts">
  import * as auth from "$lib/auth.svelte.ts";
  import { api, uploadFile as apiUpload, extractErrorMessage } from "$lib/api";
  import { goto } from "$app/navigation";
  import { extractPdfCover } from "$lib/pdf";

  interface Book {
    id: string;
    title: string;
    author: string | null;
    cover_path: string | null;
    format: string;
    file_size: number | null;
    created_at: string;
  }

  const ALLOWED_EXTENSIONS = [".epub", ".pdf", ".mobi"];

  let books = $state<Book[]>([]);
  let uploading = $state(false);
  let uploadProgress = $state(0);
  let uploadStatus = $state("");
  let dragOver = $state(false);
  let error = $state("");
  let loaded = $state(false);
  let deleteId = $state<string | null>(null);
  let deleting = $state(false);

  const API_URL = import.meta.env.VITE_API_URL ?? "http://localhost:3001";

  function coverUrl(book: Book): string | null {
    if (book.cover_path) {
      return `${API_URL}/api/books/${book.id}/cover`;
    }
    return null;
  }

  function isValidFile(file: File): boolean {
    const lower = file.name.toLowerCase();
    return ALLOWED_EXTENSIONS.some((ext) => lower.endsWith(ext));
  }

  function detectFormat(file: File): string {
    const lower = file.name.toLowerCase();
    if (lower.endsWith(".epub")) return "epub";
    if (lower.endsWith(".pdf")) return "pdf";
    if (lower.endsWith(".mobi")) return "mobi";
    return "epub";
  }

  async function loadBooks() {
    try {
      const res = await api("/api/books");
      if (res.ok) {
        books = await res.json();
        console.log("[library] loaded books:", books.length);
      } else {
        console.warn("[library] list failed:", res.status, res.statusText);
      }
    } catch (e) {
      console.error("[library] list error:", e);
    } finally {
      loaded = true;
    }
  }

  $effect(() => {
    loadBooks();
  });

  async function uploadFile(file: File) {
    if (!isValidFile(file)) {
      error = `Unsupported file type. Allowed: ${ALLOWED_EXTENSIONS.join(", ")}`;
      return;
    }

    uploading = true;
    uploadProgress = 0;
    uploadStatus = "Preparing...";
    error = "";
    try {
      const form = new FormData();
      form.append("file", file);

      const format = detectFormat(file);
      if (format === "pdf") {
        uploadStatus = "Rendering cover...";
        const coverBlob = await extractPdfCover(file);
        if (coverBlob) {
          form.append("cover", coverBlob, "cover.jpg");
        }
      }

      uploadStatus = "Uploading...";
      const res = await apiUpload("/api/books", form, (loaded, total) => {
        uploadProgress = Math.round((loaded / total) * 100);
      });

      if (res.ok) {
        await loadBooks();
      } else {
        error = await extractErrorMessage(res);
      }
    } catch (e) {
      error = e instanceof Error ? e.message : "Upload failed";
    } finally {
      uploading = false;
      uploadProgress = 0;
      uploadStatus = "";
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

  async function confirmDelete() {
    if (!deleteId) return;
    deleting = true;
    try {
      const res = await api(`/api/books/${deleteId}`, { method: "DELETE" });
      if (res.ok) {
        books = books.filter((b) => b.id !== deleteId);
      }
    } catch {
      // ignore
    } finally {
      deleting = false;
      deleteId = null;
    }
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
        <p>Drag and drop an EPUB, PDF, or MOBI file or click to browse.</p>
        <label class="upload-area" class:dragover={dragOver}>
          <input
            type="file"
            accept=".epub,.pdf,.mobi"
            onchange={handleFileInput}
            disabled={uploading}
          />
          {#if uploading}
            <div class="upload-progress">
              <div class="upload-progress-bar">
                <div class="upload-progress-fill" style="width: {uploadProgress}%"></div>
              </div>
              <span class="upload-progress-text">{uploadStatus} {uploadProgress}%</span>
            </div>
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
        <div class="library-header">
          <label class="upload-btn">
            <input
              type="file"
              accept=".epub,.pdf,.mobi"
              onchange={handleFileInput}
              disabled={uploading}
            />
            {uploading ? `${uploadStatus} ${uploadProgress}%` : "+ Add book"}
          </label>
          {#if uploading}
            <div class="upload-progress upload-progress-inline">
              <div class="upload-progress-bar">
                <div class="upload-progress-fill" style="width: {uploadProgress}%"></div>
              </div>
            </div>
          {/if}
        </div>
        {#if error}
          <p class="error">{error}</p>
        {/if}
        <div class="book-grid">
          {#each books as book (book.id)}
            <div class="book-card" role="link" tabindex="0">
              <button class="book-card-link" onclick={() => goto(`/read/${book.id}`)}>
                <div class="book-cover" class:epub={book.format === "epub"} class:pdf={book.format === "pdf"} class:mobi={book.format === "mobi"}>
                  {#if coverUrl(book)}
                    <img src={coverUrl(book)!} alt={book.title} class="cover-img" />
                  {:else}
                    <span class="cover-format">{book.format.toUpperCase()}</span>
                  {/if}
                </div>
                <div class="book-info">
                  <p class="book-title">{book.title}</p>
                  {#if book.author}
                    <p class="book-author">{book.author}</p>
                  {/if}
                  <p class="book-meta">{formatSize(book.file_size)}</p>
                </div>
              </button>
              <button
                class="delete-btn"
                title="Delete book"
                onclick={(e) => {
                  e.stopPropagation();
                  deleteId = book.id;
                }}
              >&times;</button>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </main>
</div>

{#if deleteId}
  <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
  <div class="modal-overlay" role="dialog" aria-modal="true" onclick={() => (deleteId = null)}>
    <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
    <div class="modal" onclick={(e) => e.stopPropagation()}>
      <p>Delete this book?</p>
      <p class="modal-sub">This cannot be undone.</p>
      <div class="modal-actions">
        <button class="modal-cancel" onclick={() => (deleteId = null)}>Cancel</button>
        <button class="modal-confirm" onclick={confirmDelete} disabled={deleting}>
          {deleting ? "Deleting..." : "Delete"}
        </button>
      </div>
    </div>
  </div>
{/if}

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

  .library-header {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 1rem;
    flex-wrap: wrap;
  }

  .upload-progress {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    color: #666;
    font-size: 0.9rem;
    min-width: 200px;
  }
  .upload-progress-inline {
    flex: 1;
    max-width: 300px;
    flex-direction: row;
    align-items: center;
    gap: 0.5rem;
  }
  .upload-progress-bar {
    flex: 1;
    height: 6px;
    background: #e5e7eb;
    border-radius: 3px;
    overflow: hidden;
  }
  .upload-progress-fill {
    height: 100%;
    background: #4f46e5;
    transition: width 0.15s;
  }
  .upload-progress-text {
    font-size: 0.85rem;
    color: #666;
  }
  .book-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: 1rem;
  }
  .book-card {
    position: relative;
    border-radius: 8px;
    overflow: visible;
    transition: transform 0.15s, box-shadow 0.15s;
  }
  .book-card:hover {
    transform: translateY(-2px);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
  }
  .book-card-link {
    all: unset;
    display: block;
    width: 100%;
    cursor: pointer;
    border-radius: 8px;
    overflow: hidden;
  }
  .book-cover {
    position: relative;
    width: 100%;
    padding-bottom: 150%;
    border-radius: 6px;
    overflow: hidden;
    background: #e5e7eb;
  }
  .book-cover.epub {
    background: linear-gradient(135deg, #e0c3fc, #8ec5fc);
  }
  .book-cover.pdf {
    background: linear-gradient(135deg, #fcc3c3, #fc8e8e);
  }
  .book-cover.mobi {
    background: linear-gradient(135deg, #c3fce0, #8efcb0);
  }
  .cover-img {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }
  .cover-format {
    position: absolute;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
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

  .delete-btn {
    position: absolute;
    top: -4px;
    right: -4px;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    border: 1px solid #d1d5db;
    background: #fff;
    color: #999;
    font-size: 0.9rem;
    cursor: pointer;
    display: none;
    align-items: center;
    justify-content: center;
    line-height: 1;
    z-index: 1;
    transition: color 0.15s, border-color 0.15s;
    padding: 0;
  }
  .book-card:hover .delete-btn {
    display: flex;
  }
  .delete-btn:hover {
    color: #dc2626;
    border-color: #dc2626;
  }

  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.3);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 100;
  }
  .modal {
    background: #fff;
    border-radius: 12px;
    padding: 1.5rem;
    min-width: 280px;
    text-align: center;
    box-shadow: 0 8px 30px rgba(0, 0, 0, 0.15);
  }
  .modal p {
    margin: 0 0 0.25rem;
    font-size: 0.95rem;
    color: #111;
  }
  .modal-sub {
    color: #888 !important;
    font-size: 0.85rem !important;
    margin-bottom: 1rem !important;
  }
  .modal-actions {
    display: flex;
    gap: 0.5rem;
    justify-content: center;
    margin-top: 1rem;
  }
  .modal-cancel {
    padding: 0.4rem 1rem;
    border: 1px solid #d1d5db;
    border-radius: 6px;
    background: #fff;
    cursor: pointer;
    font-size: 0.85rem;
  }
  .modal-confirm {
    padding: 0.4rem 1rem;
    border: none;
    border-radius: 6px;
    background: #dc2626;
    color: #fff;
    cursor: pointer;
    font-size: 0.85rem;
  }
  .modal-confirm:disabled {
    opacity: 0.6;
    cursor: default;
  }
</style>
