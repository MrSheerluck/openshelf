<script lang="ts">
  import * as auth from "$lib/auth.svelte.ts";
  import { api, uploadFile as apiUpload, extractErrorMessage, fetchStats, fetchTags, setBookTags, updateBook, fetchBooks } from "$lib/api";
  import { goto } from "$app/navigation";
  import type { Book, Stats, TagCount } from "$lib/reader/types";

  const ALLOWED_EXTENSIONS = [".epub", ".mobi"];
  const API_URL = import.meta.env.VITE_API_URL ?? "http://localhost:3001";

  let books = $state<Book[]>([]);
  let stats = $state<Stats | null>(null);
  let allTags = $state<TagCount[]>([]);
  let uploading = $state(false);
  let uploadProgress = $state(0);
  let uploadStatus = $state("");
  let dragOver = $state(false);
  let error = $state("");
  let loaded = $state(false);
  let deleteId = $state<string | null>(null);
  let deleting = $state(false);

  let searchQuery = $state("");
  let filterStatus = $state("");
  let filterFormat = $state("");
  let filterTag = $state("");
  let sortBy = $state("created_at");
  let sortOrder = $state("desc");
  let currentView = $state("all");

  let tagEditorBook = $state<string | null>(null);
  let tagEditorTags = $state<string[]>([]);
  let statusEditorBook = $state<string | null>(null);
  let statusEditorValue = $state<string>("");
  let menuBook = $state<string | null>(null);

  let continueReading = $derived(
    (currentView === "all" ? books : [])
      .filter((b) => b.last_opened_at && b.format === "epub")
      .sort((a, b) => (b.last_opened_at ?? "").localeCompare(a.last_opened_at ?? ""))
      .slice(0, 6)
  );

  let viewTitle = $derived.by(() => {
    if (searchQuery) return `Search: "${searchQuery}"`;
    switch (currentView) {
      case "reading": return "Reading";
      case "finished": return "Finished";
      case "want_to_read": return "Want to Read";
      default: return filterTag ? filterTag : "All Books";
    }
  });

  let viewCount = $derived(books.length);

  function coverUrl(book: Book): string | null {
    if (book.cover_path) return `${API_URL}/api/books/${book.id}/cover`;
    return null;
  }

  function isValidFile(file: File): boolean {
    const lower = file.name.toLowerCase();
    return ALLOWED_EXTENSIONS.some((ext) => lower.endsWith(ext));
  }

  function formatSize(bytes: number | null): string {
    if (!bytes) return "";
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(0)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  function statusLabel(s: string | null): string {
    if (s === "reading") return "Reading";
    if (s === "finished") return "Finished";
    if (s === "want_to_read") return "Want to Read";
    return "";
  }

  function navCount(key: string): number {
    if (!stats) return 0;
    switch (key) {
      case "all": return stats.total_books;
      case "reading": return stats.reading_books;
      case "finished": return stats.finished_books;
      case "want_to_read": return stats.want_to_read;
      default: return 0;
    }
  }

  function setView(view: string) {
    currentView = view;
    searchQuery = "";
    filterTag = "";
    filterFormat = "";
    if (view === "all") {
      filterStatus = "";
    } else {
      filterStatus = view;
    }
    loadAll();
  }

  function selectTag(tag: string) {
    currentView = tag;
    filterTag = tag;
    filterStatus = "";
    searchQuery = "";
    loadAll();
  }

  async function loadAll() {
    loaded = false;
    const params: Record<string, string> = {};
    if (searchQuery) params.search = searchQuery;
    if (filterStatus) params.status = filterStatus;
    if (filterFormat) params.format = filterFormat;
    if (filterTag) params.tag = filterTag;
    if (sortBy) params.sort = sortBy;
    if (sortOrder) params.order = sortOrder;

    const [b, s, t] = await Promise.all([
      fetchBooks(params),
      fetchStats(),
      fetchTags(),
    ]);
    books = b;
    stats = s;
    allTags = t;
    loaded = true;
  }

  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  $effect(() => {
    loadAll();
  });

  function onSearchInput() {
    currentView = "all";
    filterStatus = "";
    filterTag = "";
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => loadAll(), 300);
  }

  function onFilterChange() {
    loadAll();
  }

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
      uploadStatus = "Uploading...";
      const res = await apiUpload("/api/books", form, (loaded, total) => {
        uploadProgress = Math.round((loaded / total) * 100);
      });
      if (res.ok) {
        await loadAll();
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
        stats = await fetchStats();
      }
    } catch {}
    finally {
      deleting = false;
      deleteId = null;
    }
  }

  function openTagEditor(book: Book) {
    tagEditorBook = book.id;
    tagEditorTags = [...book.tags];
  }

  async function saveTagEditor() {
    if (!tagEditorBook) return;
    const newTags = await setBookTags(tagEditorBook, tagEditorTags);
    const book = books.find((b) => b.id === tagEditorBook);
    if (book) book.tags = newTags;
    allTags = await fetchTags();
    tagEditorBook = null;
  }

  function openStatusEditor(book: Book) {
    statusEditorBook = book.id;
    statusEditorValue = book.reading_status ?? "";
  }

  async function saveStatusEditor() {
    if (!statusEditorBook) return;
    await updateBook(statusEditorBook, { reading_status: statusEditorValue });
    const book = books.find((b) => b.id === statusEditorBook);
    if (book) book.reading_status = statusEditorValue || null;
    stats = await fetchStats();
    statusEditorBook = null;
  }

  function toggleTagInEditor(tag: string) {
    if (tagEditorTags.includes(tag)) {
      tagEditorTags = tagEditorTags.filter((t) => t !== tag);
    } else {
      tagEditorTags = [...tagEditorTags, tag];
    }
  }
</script>

<svelte:head>
  <title>{viewTitle} — OpenShelf</title>
</svelte:head>

<div class="layout">
  <aside class="sidebar">
    <div class="sidebar-header">
      <h1 class="sidebar-logo">OpenShelf</h1>
    </div>

    <label class="sidebar-upload">
      <input type="file" accept=".epub,.mobi" onchange={handleFileInput} disabled={uploading} />
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><line x1="12" y1="5" x2="12" y2="19"/><line x1="5" y1="12" x2="19" y2="12"/></svg>
      {uploading ? `${uploadProgress}%` : "Upload Book"}
    </label>
    {#if uploading}
      <div class="sidebar-upload-progress">
        <div class="sidebar-upload-fill" style="width: {uploadProgress}%"></div>
      </div>
    {/if}

    <nav class="sidebar-nav">
      <button class="nav-item" class:active={currentView === "all" && !searchQuery} onclick={() => setView("all")}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M4 19.5A2.5 2.5 0 016.5 17H20"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 014 19.5v-15A2.5 2.5 0 016.5 2z"/></svg>
        <span class="nav-label">All Books</span>
        {#if stats}
          <span class="nav-count">{stats.total_books}</span>
        {/if}
      </button>
      <button class="nav-item" class:active={currentView === "reading"} onclick={() => setView("reading")}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/></svg>
        <span class="nav-label">Reading</span>
        {#if stats && stats.reading_books > 0}
          <span class="nav-count">{stats.reading_books}</span>
        {/if}
      </button>
      <button class="nav-item" class:active={currentView === "finished"} onclick={() => setView("finished")}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><polyline points="20 6 9 17 4 12"/></svg>
        <span class="nav-label">Finished</span>
        {#if stats && stats.finished_books > 0}
          <span class="nav-count">{stats.finished_books}</span>
        {/if}
      </button>
      <button class="nav-item" class:active={currentView === "want_to_read"} onclick={() => setView("want_to_read")}>
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z"/></svg>
        <span class="nav-label">Want to Read</span>
        {#if stats && stats.want_to_read > 0}
          <span class="nav-count">{stats.want_to_read}</span>
        {/if}
      </button>
    </nav>

    {#if allTags.length > 0}
      <div class="sidebar-section">
        <div class="sidebar-section-title">Tags</div>
        <div class="sidebar-tags">
          {#each allTags as tag}
            <button
              class="nav-item nav-item-tag"
              class:active={filterTag === tag.name}
              onclick={() => selectTag(tag.name)}
            >
              <span class="nav-label"># {tag.name}</span>
              <span class="nav-count">{tag.count}</span>
            </button>
          {/each}
        </div>
      </div>
    {/if}

    <div class="sidebar-footer">
      {#if stats}
        <span class="sidebar-footer-stat">{stats.total_books} books</span>
        <span class="sidebar-footer-dot"></span>
        <span class="sidebar-footer-stat">{stats.total_highlights} highlights</span>
      {/if}
    </div>
  </aside>

  <main class="main">
    <header class="main-header">
      <div class="main-header-left">
        <h2 class="main-title">{viewTitle}</h2>
        <span class="main-count">{viewCount}</span>
      </div>
      <div class="main-header-right">
        <div class="main-search">
          <svg class="main-search-icon" width="15" height="15" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="11" cy="11" r="8"/><line x1="21" y1="21" x2="16.65" y2="16.65"/></svg>
          <input type="search" placeholder="Search title or author..." bind:value={searchQuery} oninput={onSearchInput} />
        </div>
        <select class="main-sort" bind:value={sortBy} onchange={onFilterChange}>
          <option value="created_at">Newest</option>
          <option value="title">Title</option>
          <option value="author">Author</option>
          <option value="updated_at">Recent</option>
          <option value="progress">Progress</option>
        </select>
        <button class="main-order" onclick={() => { sortOrder = sortOrder === "desc" ? "asc" : "desc"; onFilterChange(); }}>
          {sortOrder === "desc" ? "\u2193" : "\u2191"}
        </button>
      </div>
    </header>

    {#if !loaded}
      <div class="loading">
        <div class="loading-bar"></div>
      </div>
    {:else if error}
      <div class="error">{error}</div>
    {:else if books.length === 0 && !searchQuery}
      <div class="empty">
        {#if currentView === "all"}
          <div class="empty-icon">
            <svg width="40" height="40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.2"><path d="M4 19.5A2.5 2.5 0 016.5 17H20"/><path d="M6.5 2H20v20H6.5A2.5 2.5 0 014 19.5v-15A2.5 2.5 0 016.5 2z"/></svg>
          </div>
          <h3>No books yet</h3>
          <p>Upload an EPUB or MOBI to get started.</p>
        {:else}
          <h3>No books here</h3>
          <p>Change your filter or upload a new book.</p>
        {/if}
      </div>
    {:else}
      {#if continueReading.length > 0}
        <section class="continue-section">
          <div class="continue-label">Continue Reading</div>
          <div class="continue-row">
            {#each continueReading as book (book.id)}
              <button class="continue-card" onclick={() => goto(`/read/${book.id}`)}>
                <div class="continue-cover">
                  {#if coverUrl(book)}
                    <img src={coverUrl(book)!} alt={book.title} />
                  {:else}
                    <span class="continue-placeholder">{book.format.toUpperCase()}</span>
                  {/if}
                  {#if book.progress != null && book.progress > 0}
                    <div class="continue-progress" style="width: {Math.min(book.progress, 100)}%"></div>
                  {/if}
                </div>
                <span class="continue-title">{book.title}</span>
              </button>
            {/each}
          </div>
        </section>
      {/if}

      <div class="grid">
        {#each books as book (book.id)}
          <div class="card" role="button" tabindex="0" onclick={() => { if (!menuBook) goto(`/read/${book.id}`); }} onkeydown={(e) => { if (e.key === 'Enter') goto(`/read/${book.id}`); }}>
            <div class="card-cover">
              {#if coverUrl(book)}
                <img src={coverUrl(book)!} alt={book.title} />
              {:else}
                <div class="card-placeholder">{book.format.toUpperCase()}</div>
              {/if}
              {#if book.progress != null && book.progress > 0}
                <div class="card-progress" style="width: {Math.min(book.progress, 100)}%"></div>
              {/if}
              <button class="card-menu-btn" onclick={(e) => { e.stopPropagation(); menuBook = menuBook === book.id ? null : book.id; }}>
                <svg width="15" height="15" viewBox="0 0 24 24" fill="currentColor"><circle cx="12" cy="5" r="1.5"/><circle cx="12" cy="12" r="1.5"/><circle cx="12" cy="19" r="1.5"/></svg>
              </button>
              {#if menuBook === book.id}
                <div class="card-menu" onclick={(e) => e.stopPropagation()}>
                  <button onclick={() => { menuBook = null; openStatusEditor(book); }}>
                    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/></svg>
                    {book.reading_status ? statusLabel(book.reading_status) : "Set status"}
                  </button>
                  <button onclick={() => { menuBook = null; openTagEditor(book); }}>
                    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M20.59 13.41l-7.17 7.17a2 2 0 01-2.83 0L2 12V2h10l8.59 8.59a2 2 0 010 2.82z"/><line x1="7" y1="7" x2="7.01" y2="7"/></svg>
                    {book.tags.length ? `Tags (${book.tags.length})` : "Add tags"}
                  </button>
                  <div class="card-menu-sep"></div>
                  <button class="menu-danger" onclick={() => { menuBook = null; deleteId = book.id; }}>
                    <svg width="13" height="13" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="3 6 5 6 21 6"/><path d="M19 6v14a2 2 0 01-2 2H7a2 2 0 01-2-2V6m3 0V4a2 2 0 012-2h4a2 2 0 012 2v2"/></svg>
                    Delete
                  </button>
                </div>
              {/if}
            </div>
            <div class="card-body">
              <div class="card-title-row">
                <span class="card-title">{book.title}</span>
                {#if book.reading_status}
                  <span class="card-dot" class:dot-green={book.reading_status === "reading"} class:dot-blue={book.reading_status === "finished"} class:dot-amber={book.reading_status === "want_to_read"}></span>
                {/if}
              </div>
              {#if book.author}
                <span class="card-author">{book.author}</span>
              {/if}
              {#if book.tags.length > 0}
                <div class="card-tags">
                  {#each book.tags.slice(0, 2) as tag}
                    <span class="card-tag">{tag}</span>
                  {/each}
                  {#if book.tags.length > 2}
                    <span class="card-tag-more">+{book.tags.length - 2}</span>
                  {/if}
                </div>
              {/if}
            </div>
          </div>
        {/each}
      </div>
    {/if}
  </main>
</div>

{#if deleteId}
  <div class="overlay" onclick={() => (deleteId = null)}>
    <div class="dialog" onclick={(e) => e.stopPropagation()}>
      <h3>Remove this book?</h3>
      <p class="dialog-desc">Highlights and bookmarks will also be deleted.</p>
      <div class="dialog-actions">
        <button class="btn" onclick={() => (deleteId = null)}>Cancel</button>
        <button class="btn btn-danger" onclick={confirmDelete} disabled={deleting}>
          {deleting ? "Deleting..." : "Delete"}
        </button>
      </div>
    </div>
  </div>
{/if}

{#if tagEditorBook}
  <div class="overlay" onclick={() => (tagEditorBook = null)}>
    <div class="dialog dialog-tags" onclick={(e) => e.stopPropagation()}>
      <h3>Edit tags</h3>
      <div class="tag-input-row">
        <input type="text" placeholder="Type and press Enter..." onkeydown={(e) => {
          if (e.key === "Enter") {
            const input = e.target as HTMLInputElement;
            const val = input.value.trim();
            if (val && !tagEditorTags.includes(val)) tagEditorTags = [...tagEditorTags, val];
            input.value = "";
          }
        }} />
      </div>
      <div class="tag-chips">
        {#each tagEditorTags as tag}
          <span class="tag-chip">
            {tag}
            <button class="tag-chip-x" onclick={() => (tagEditorTags = tagEditorTags.filter((t) => t !== tag))}>&times;</button>
          </span>
        {/each}
      </div>
      {#if allTags.length > 0}
        <div class="tag-suggest-label">All tags</div>
        <div class="tag-suggestions">
          {#each allTags as t}
            <button class="tag-sug" class:active={tagEditorTags.includes(t.name)} onclick={() => toggleTagInEditor(t.name)}>
              {t.name}
              <span class="tag-sug-count">{t.count}</span>
            </button>
          {/each}
        </div>
      {/if}
      <div class="dialog-actions">
        <button class="btn" onclick={() => (tagEditorBook = null)}>Cancel</button>
        <button class="btn btn-primary" onclick={saveTagEditor}>Save</button>
      </div>
    </div>
  </div>
{/if}

{#if statusEditorBook}
  <div class="overlay" onclick={() => (statusEditorBook = null)}>
    <div class="dialog" onclick={(e) => e.stopPropagation()}>
      <h3>Reading status</h3>
      <div class="status-opts">
        <button class="status-opt" class:active={statusEditorValue === ""} onclick={() => (statusEditorValue = "")}>None</button>
        <button class="status-opt" class:active={statusEditorValue === "reading"} class:opt-green={statusEditorValue === "reading"} onclick={() => (statusEditorValue = "reading")}>
          <span class="status-dot dot-green"></span> Reading
        </button>
        <button class="status-opt" class:active={statusEditorValue === "finished"} class:opt-blue={statusEditorValue === "finished"} onclick={() => (statusEditorValue = "finished")}>
          <span class="status-dot dot-blue"></span> Finished
        </button>
        <button class="status-opt" class:active={statusEditorValue === "want_to_read"} class:opt-amber={statusEditorValue === "want_to_read"} onclick={() => (statusEditorValue = "want_to_read")}>
          <span class="status-dot dot-amber"></span> Want to Read
        </button>
      </div>
      <div class="dialog-actions">
        <button class="btn" onclick={() => (statusEditorBook = null)}>Cancel</button>
        <button class="btn btn-primary" onclick={saveStatusEditor}>Save</button>
      </div>
    </div>
  </div>
{/if}

<style>
  @import '@fontsource/literata/400.css';
  @import '@fontsource/literata/500.css';
  @import '@fontsource/literata/600.css';
  @import '@fontsource/literata/700.css';

  :root {
    --sidebar: #f1f3f5;
    --sidebar-hover: #e8eaed;
    --sidebar-active: #fff;
    --accent: #1a56db;
    --accent-bg: #eef2ff;
    --text: #16181a;
    --text-muted: #6b7280;
    --text-faint: #9ca3af;
    --border: #e2e4e8;
    --surface: #fff;
    --bg: #f8f9fa;
    --radius: 8px;
    --radius-sm: 6px;
    --shadow: 0 1px 3px rgba(0,0,0,0.05);
    --shadow-hover: 0 4px 14px rgba(0,0,0,0.08);
    --green: #16a34a; --green-bg: #f0fdf4;
    --blue: #2563eb; --blue-bg: #eff6ff;
    --amber: #d97706; --amber-bg: #fffbeb;
    --danger: #dc2626; --danger-bg: #fef2f2;
    --font-display: "Literata", Georgia, "Times New Roman", serif;
    --font-ui: system-ui, -apple-system, sans-serif;
  }

  * { box-sizing: border-box; }
  html, body { margin: 0; }

  .layout {
    display: flex;
    height: 100vh;
    font-family: var(--font-ui);
    -webkit-font-smoothing: antialiased;
    background: var(--bg);
    color: var(--text);
  }

  /* ── Sidebar ─────────────────────────── */

  .sidebar {
    width: 220px;
    flex-shrink: 0;
    background: var(--sidebar);
    border-right: 1px solid var(--border);
    display: flex;
    flex-direction: column;
    padding: 0;
    overflow-y: auto;
  }
  .sidebar::-webkit-scrollbar { width: 6px; }
  .sidebar::-webkit-scrollbar-track { background: transparent; }
  .sidebar::-webkit-scrollbar-thumb { background: var(--border); border-radius: 3px; }
  .sidebar-header {
    padding: 1rem 1rem 0.5rem;
  }
  .sidebar-logo {
    font-family: var(--font-display);
    font-size: 1.15rem;
    font-weight: 700;
    margin: 0;
    letter-spacing: -0.01em;
    color: var(--text);
  }
  .sidebar-upload {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    margin: 0.25rem 0.75rem;
    padding: 0.45rem 0.65rem;
    border-radius: var(--radius-sm);
    border: 1px dashed var(--border);
    cursor: pointer;
    font-size: 0.8rem;
    color: var(--text-muted);
    transition: all 0.12s;
  }
  .sidebar-upload:hover {
    border-color: var(--accent);
    color: var(--accent);
    background: var(--accent-bg);
  }
  .sidebar-upload input { display: none; }
  .sidebar-upload-progress {
    height: 3px;
    margin: 0 0.75rem;
    background: var(--border);
    border-radius: 2px;
    overflow: hidden;
  }
  .sidebar-upload-fill {
    height: 100%;
    background: var(--accent);
    transition: width 0.2s;
  }

  .sidebar-nav {
    display: flex;
    flex-direction: column;
    gap: 1px;
    padding: 0.35rem 0.5rem;
  }
  .nav-item {
    display: flex;
    align-items: center;
    gap: 0.45rem;
    padding: 0.45rem 0.55rem;
    border: none;
    border-radius: var(--radius-sm);
    background: none;
    cursor: pointer;
    font-size: 0.82rem;
    font-family: inherit;
    color: var(--text-muted);
    text-align: left;
    transition: all 0.1s;
    width: 100%;
  }
  .nav-item:hover { background: var(--sidebar-hover); color: var(--text); }
  .nav-item.active {
    background: var(--sidebar-active);
    color: var(--text);
    font-weight: 500;
    box-shadow: var(--shadow);
  }
  .nav-item svg { flex-shrink: 0; }
  .nav-label { font-family: var(--font-display); font-size: 0.84rem; flex: 1; min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .nav-count {
    font-size: 0.7rem;
    padding: 0.1rem 0.4rem;
    border-radius: 10px;
    background: var(--border);
    color: var(--text-muted);
    font-weight: 500;
  }
  .nav-item.active .nav-count { background: var(--accent-bg); color: var(--accent); }

  .sidebar-section {
    padding: 0.5rem;
    border-top: 1px solid var(--border);
    margin-top: auto;
  }
  .sidebar-section-title {
    font-size: 0.68rem;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-faint);
    padding: 0.3rem 0.55rem;
  }
  .sidebar-tags {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }
  .nav-item-tag .nav-label { color: var(--text-muted); }

  .sidebar-footer {
    padding: 0.75rem 1rem;
    border-top: 1px solid var(--border);
    display: flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.72rem;
    color: var(--text-faint);
    margin-top: auto;
  }
  .sidebar-footer-dot {
    width: 3px;
    height: 3px;
    border-radius: 50%;
    background: var(--text-faint);
  }

  /* ── Main ────────────────────────────── */

  .main {
    flex: 1;
    overflow-y: auto;
    padding: 0;
  }
  .main-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 1rem 1.5rem;
    gap: 1rem;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
    position: sticky;
    top: 0;
    z-index: 10;
  }
  .main-header-left {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-width: 0;
  }
  .main-title {
    font-family: var(--font-display);
    font-size: 1.1rem;
    font-weight: 600;
    margin: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    letter-spacing: -0.01em;
  }
  .main-count {
    font-size: 0.78rem;
    color: var(--text-faint);
    background: var(--bg);
    padding: 0.1rem 0.45rem;
    border-radius: 10px;
    white-space: nowrap;
  }
  .main-header-right {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    flex-shrink: 0;
  }
  .main-search {
    position: relative;
  }
  .main-search-icon {
    position: absolute;
    left: 8px;
    top: 50%;
    transform: translateY(-50%);
    color: var(--text-faint);
    pointer-events: none;
  }
  .main-search input {
    width: 200px;
    padding: 0.4rem 0.65rem 0.4rem 1.75rem;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    font-size: 0.8rem;
    font-family: inherit;
    background: var(--bg);
    color: var(--text);
    outline: none;
    transition: border-color 0.12s;
  }
  .main-search input:focus { border-color: var(--accent); }
  .main-search input::placeholder { color: var(--text-faint); }

  .main-sort {
    padding: 0.35rem 0.5rem;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    font-size: 0.78rem;
    font-family: inherit;
    background: var(--surface);
    color: var(--text-muted);
    cursor: pointer;
    outline: none;
  }
  .main-sort:focus { border-color: var(--accent); }
  .main-order {
    padding: 0.35rem 0.4rem;
    border: 1px solid var(--border);
    border-radius: var(--radius-sm);
    background: var(--surface);
    cursor: pointer;
    font-size: 0.85rem;
    color: var(--text-muted);
    line-height: 1;
  }
  .main-order:hover { border-color: var(--text-faint); color: var(--text); }

  /* ── Loading & Empty ─────────────────── */

  .loading {
    display: flex;
    justify-content: center;
    padding: 3rem;
  }
  .loading-bar {
    width: 160px;
    height: 3px;
    border-radius: 2px;
    background: var(--border);
    overflow: hidden;
    position: relative;
  }
  .loading-bar::after {
    content: '';
    position: absolute;
    left: -40%;
    width: 40%;
    height: 100%;
    background: var(--accent);
    border-radius: 2px;
    animation: load-slide 1.2s ease-in-out infinite;
  }
  @keyframes load-slide {
    0% { left: -40%; }
    100% { left: 100%; }
  }
  .error { padding: 1rem 1.5rem; color: var(--danger); font-size: 0.85rem; }
  .empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 4rem 2rem;
    color: var(--text-muted);
    text-align: center;
  }
  .empty-icon { color: var(--text-faint); margin-bottom: 0.75rem; }
  .empty h3 { font-family: var(--font-display); font-size: 1rem; font-weight: 500; margin: 0 0 0.25rem; color: var(--text); }
  .empty p { font-size: 0.85rem; margin: 0; }

  /* ── Continue Reading ────────────────── */

  .continue-section {
    padding: 1rem 1.5rem 0.5rem;
  }
  .continue-label {
    font-family: var(--font-display);
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--text);
    margin-bottom: 0.5rem;
    letter-spacing: -0.01em;
  }
  .continue-row {
    display: flex;
    gap: 0.65rem;
    overflow-x: auto;
    padding: 0.15rem 0.15rem 0.35rem;
    scrollbar-width: thin;
  }
  .continue-card {
    all: unset;
    flex: 0 0 100px;
    cursor: pointer;
    border-radius: var(--radius-sm);
    overflow: hidden;
    transition: transform 0.15s;
  }
  .continue-card:hover { transform: translateY(-2px); }
  .continue-cover {
    position: relative;
    width: 100px;
    height: 150px;
    border-radius: var(--radius-sm);
    overflow: hidden;
    background: var(--border);
    box-shadow: var(--shadow);
  }
  .continue-cover img { width: 100%; height: 100%; object-fit: cover; display: block; }
  .continue-placeholder {
    display: flex; align-items: center; justify-content: center; height: 100%;
    font-size: 0.7rem; font-weight: 600; color: var(--text-faint); letter-spacing: 0.04em;
  }
  .continue-progress {
    position: absolute; bottom: 0; left: 0; height: 3px;
    background: var(--accent); transition: width 0.3s; border-radius: 0 2px 0 0;
  }
  .continue-title {
    font-family: var(--font-display);
    display: block; font-size: 0.76rem; font-weight: 500; color: var(--text); margin-top: 0.3rem;
    white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
  }

  /* ── Grid ────────────────────────────── */

  .grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(150px, 1fr));
    gap: 0.75rem;
    padding: 0.75rem 1.5rem 2rem;
  }
  .card {
    position: relative;
    border-radius: var(--radius-sm);
    overflow: hidden;
    background: var(--surface);
    cursor: pointer;
    transition: transform 0.15s, box-shadow 0.15s;
  }
  .card:hover { transform: translateY(-2px); box-shadow: var(--shadow-hover); }
  .card:active { transform: translateY(0); }

  .card-cover {
    position: relative;
    width: 100%;
    padding-bottom: 148%;
    background: var(--border);
    overflow: hidden;
  }
  .card-cover img {
    position: absolute;
    top: 0; left: 0;
    width: 100%; height: 100%;
    object-fit: cover;
    display: block;
    transition: transform 0.25s;
  }
  .card:hover .card-cover img { transform: scale(1.04); }
  .card-placeholder {
    position: absolute; inset: 0;
    display: flex; align-items: center; justify-content: center;
    font-size: 0.8rem; font-weight: 600; color: var(--text-faint); letter-spacing: 0.04em;
  }
  .card-progress {
    position: absolute; bottom: 0; left: 0; height: 3px;
    background: var(--accent); transition: width 0.35s; z-index: 1;
  }

  .card-menu-btn {
    position: absolute; top: 5px; right: 5px;
    width: 26px; height: 26px; border-radius: 50%; border: none;
    background: rgba(255,255,255,0.88);
    color: var(--text-muted); cursor: pointer;
    display: flex; align-items: center; justify-content: center;
    opacity: 0; transition: opacity 0.12s; z-index: 2;
    backdrop-filter: blur(4px);
  }
  .card:hover .card-menu-btn, .card-menu-btn:focus-visible { opacity: 1; }
  .card-menu-btn:hover { background: #fff; color: var(--text); }

  .card-menu {
    position: absolute; top: 34px; right: 5px;
    background: var(--surface); border: 1px solid var(--border);
    border-radius: var(--radius-sm); box-shadow: 0 8px 24px rgba(0,0,0,0.12);
    z-index: 10; min-width: 145px; overflow: hidden; padding: 0.2rem;
  }
  .card-menu button {
    display: flex; align-items: center; gap: 0.4rem;
    width: 100%; padding: 0.4rem 0.5rem;
    border: none; background: none; cursor: pointer;
    font-size: 0.76rem; font-family: inherit; color: var(--text);
    border-radius: 4px; text-align: left; transition: background 0.08s;
  }
  .card-menu button:hover { background: var(--bg); }
  .card-menu button svg { flex-shrink: 0; color: var(--text-muted); }
  .card-menu-sep { height: 1px; background: var(--border); margin: 0.15rem 0.3rem; }
  .card-menu .menu-danger { color: var(--danger); }
  .card-menu .menu-danger:hover { background: var(--danger-bg); }

  .card-body { padding: 0.45rem 0.5rem 0.5rem; }
  .card-title-row { display: flex; align-items: center; gap: 0.3rem; }
  .card-title { font-family: var(--font-display); font-size: 0.82rem; font-weight: 500; color: var(--text); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; min-width: 0; }
  .card-dot { width: 6px; height: 6px; border-radius: 50%; flex-shrink: 0; }
  .dot-green { background: var(--green); }
  .dot-blue { background: var(--blue); }
  .dot-amber { background: var(--amber); }
  .card-author { display: block; font-size: 0.72rem; color: var(--text-muted); margin-top: 0.1rem; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .card-tags { display: flex; gap: 0.2rem; align-items: center; margin-top: 0.2rem; overflow: hidden; }
  .card-tag { font-size: 0.62rem; padding: 0.05rem 0.25rem; border-radius: 3px; background: var(--bg); color: var(--text-faint); border: 1px solid var(--border); white-space: nowrap; }
  .card-tag-more { font-size: 0.62rem; color: var(--text-faint); }

  /* ── Overlay & Dialogs ───────────────── */

  .overlay {
    position: fixed; inset: 0;
    background: rgba(0,0,0,0.35);
    backdrop-filter: blur(2px);
    display: flex; align-items: center; justify-content: center;
    z-index: 100; padding: 1rem;
  }
  .dialog {
    background: var(--surface); border-radius: 10px; padding: 1.25rem;
    min-width: 280px; max-width: 380px; width: 100%;
    box-shadow: 0 8px 30px rgba(0,0,0,0.15);
  }
  .dialog h3 { font-family: var(--font-display); font-size: 1rem; font-weight: 600; margin: 0 0 0.35rem; color: var(--text); letter-spacing: -0.01em; }
  .dialog-desc { font-size: 0.82rem; color: var(--text-muted); margin: 0 0 1rem; line-height: 1.4; }
  .dialog-actions { display: flex; gap: 0.4rem; justify-content: flex-end; margin-top: 1rem; }
  .btn {
    padding: 0.35rem 0.85rem; border: 1px solid var(--border); border-radius: var(--radius-sm);
    background: var(--surface); cursor: pointer; font-size: 0.8rem; font-family: inherit;
    color: var(--text-muted); transition: all 0.1s;
  }
  .btn:hover { border-color: var(--text-faint); color: var(--text); }
  .btn-primary {
    border-color: var(--accent); background: var(--accent); color: #fff; font-weight: 500;
  }
  .btn-primary:hover { opacity: 0.9; }
  .btn-danger { border-color: var(--danger); background: var(--danger); color: #fff; }
  .btn-danger:disabled { opacity: 0.5; }

  /* Tag editor */
  .dialog-tags { text-align: left; }
  .tag-input-row { margin-bottom: 0.5rem; }
  .tag-input-row input {
    width: 100%; padding: 0.35rem 0.55rem;
    border: 1px solid var(--border); border-radius: var(--radius-sm);
    font-size: 0.82rem; font-family: inherit; background: var(--bg);
    color: var(--text); outline: none;
  }
  .tag-input-row input:focus { border-color: var(--accent); }
  .tag-chips { display: flex; gap: 0.25rem; flex-wrap: wrap; margin-bottom: 0.65rem; min-height: 1.3rem; }
  .tag-chip {
    display: inline-flex; align-items: center; gap: 0.2rem;
    font-size: 0.76rem; padding: 0.12rem 0.4rem; border-radius: 4px;
    background: var(--accent-bg); color: var(--accent); border: 1px solid #dbeafe;
  }
  .tag-chip-x { background: none; border: none; cursor: pointer; color: var(--accent); font-size: 0.95rem; padding: 0; line-height: 1; opacity: 0.6; }
  .tag-chip-x:hover { opacity: 1; }
  .tag-suggest-label { font-size: 0.7rem; color: var(--text-faint); margin-bottom: 0.3rem; }
  .tag-suggestions { display: flex; gap: 0.25rem; flex-wrap: wrap; margin-bottom: 0.5rem; }
  .tag-sug {
    display: inline-flex; align-items: center; gap: 0.25rem;
    font-size: 0.74rem; padding: 0.15rem 0.45rem; border-radius: 4px;
    border: 1px solid var(--border); background: var(--surface);
    cursor: pointer; color: var(--text-muted); font-family: inherit; transition: all 0.08s;
  }
  .tag-sug:hover { border-color: var(--accent); color: var(--accent); }
  .tag-sug.active { background: var(--accent-bg); border-color: var(--accent); color: var(--accent); }
  .tag-sug-count { font-size: 0.68rem; color: var(--text-faint); }

  /* Status editor */
  .status-opts { display: flex; flex-direction: column; gap: 0.3rem; }
  .status-opt {
    display: flex; align-items: center; gap: 0.45rem;
    padding: 0.45rem 0.65rem; border: 1px solid var(--border); border-radius: var(--radius-sm);
    background: var(--surface); cursor: pointer; font-size: 0.82rem; font-family: inherit;
    color: var(--text); text-align: left; transition: all 0.1s;
  }
  .status-opt:hover { border-color: var(--text-faint); }
  .status-opt.active { border-width: 1.5px; }
  .opt-green.active { border-color: var(--green); background: var(--green-bg); }
  .opt-blue.active { border-color: var(--blue); background: var(--blue-bg); }
  .opt-amber.active { border-color: var(--amber); background: var(--amber-bg); }
  .status-dot { width: 7px; height: 7px; border-radius: 50%; }
</style>