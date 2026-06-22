<script lang="ts">
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import { api } from "$lib/api";

  interface Book {
    id: string;
    title: string;
    author: string | null;
    format: string;
  }

  type Theme = "light" | "dark" | "sepia";
  const themes: { name: Theme; bg: string; fg: string }[] = [
    { name: "light", bg: "#ffffff", fg: "#1a1a1a" },
    { name: "sepia", bg: "#f4ecd8", fg: "#3a2f1c" },
    { name: "dark", bg: "#1a1a1a", fg: "#d4d4d4" },
  ];

  let book = $state<Book | null>(null);
  let loading = $state(true);
  let error = $state("");
  let viewerEl = $state<HTMLDivElement | null>(null);
  let rendition: any = null;
  let bookObj: any = null;
  let fileBlobUrl = $state<string | null>(null);
  let theme = $state<Theme>("light");
  let fontSize = $state(100);
  let showControls = $state(true);
  let showToc = $state(false);
  let toc = $state<any[]>([]);
  let progress = $state(0);
  let currentChapter = $state("");

  const API_URL = import.meta.env.VITE_API_URL ?? "http://localhost:3001";
  const id = $derived(String(page.params.id));

  function fileUrl(): string {
    return `${API_URL}/api/books/${id}/file`;
  }

  async function prefetchFileBlob(): Promise<string | null> {
    try {
      const res = await fetch(fileUrl(), { credentials: "include" });
      if (!res.ok) return null;
      const blob = await res.blob();
      if (fileBlobUrl) URL.revokeObjectURL(fileBlobUrl);
      fileBlobUrl = URL.createObjectURL(blob);
      return fileBlobUrl;
    } catch {
      return null;
    }
  }

  function getStoredSettings() {
    try {
      const stored = localStorage.getItem(`openshelf:reader:${id}`);
      if (stored) return JSON.parse(stored);
    } catch {}
    return null;
  }

  function setStoredSettings(settings: { theme: Theme; fontSize: number; cfi?: string }) {
    try {
      localStorage.setItem(`openshelf:reader:${id}`, JSON.stringify(settings));
    } catch {}
  }

  async function saveProgress(cfi: string) {
    if (!cfi) return;
    try {
      await api(`/api/books/${id}/progress`, {
        method: "POST",
        body: JSON.stringify({ cfi }),
      });
    } catch {}
  }

  async function loadBook() {
    try {
      const res = await api(`/api/books/${id}`);
      if (res.ok) {
        book = await res.json();
        loading = false;
      } else {
        error = "Book not found";
        loading = false;
      }
    } catch {
      error = "Failed to load book";
      loading = false;
    }
  }

  async function renderEpub() {
    if (!viewerEl || !book) return;
    try {
      const ePub = (await import("epubjs")).default;

      bookObj = ePub(fileUrl(), {
        openAs: "epub",
        requestMethod: async (url: string) => {
          const res = await fetch(url, { credentials: "include" });
          if (!res.ok) {
            throw new Error(`Failed to fetch book: ${res.status} ${res.statusText}`);
          }
          return await res.arrayBuffer();
        },
      });

      rendition = bookObj.renderTo(viewerEl, {
        width: "100%",
        height: "100%",
        flow: "paginated",
        spread: "none",
        manager: "default",
      });

      themes.forEach((t) => {
        rendition.themes.register(t.name, {
          "body": {
            "background": `${t.bg} !important`,
            "color": `${t.fg} !important`,
          },
          "p, div, span, li": {
            "color": `${t.fg} !important`,
          },
        });
      });

      rendition.themes.fontSize(`${fontSize}%`);

      const stored = getStoredSettings();
      if (stored) {
        theme = stored.theme ?? "light";
        fontSize = stored.fontSize ?? 100;
        rendition.themes.select(theme);
        rendition.themes.fontSize(`${fontSize}%`);
      }

      await rendition.display();

      if (stored?.cfi) {
        await rendition.display(stored.cfi);
      }

      toc = (bookObj.navigation.toc ?? []).map((item: any) => ({
        label: item.label,
        href: item.href,
        subitems: item.subitems ?? [],
      }));

      bookObj.ready.then(async () => {
        if (bookObj.locations.length() === 0) {
          await bookObj.locations.generate(1024);
        }
      });

      rendition.on("relocated", (location: any) => {
        progress = Math.round((location.start.percentage ?? 0) * 100);
        currentChapter = location.start.href ?? "";
        const cfi = location.start.cfi;
        if (cfi) {
          setStoredSettings({ theme, fontSize, cfi });
          saveProgress(cfi);
        }
      });
    } catch (e) {
      console.error("EPUB render error:", e);
      error = `Failed to render book: ${e instanceof Error ? e.message : String(e)}`;
      loading = false;
    }
  }

  $effect(() => {
    loadBook();
  });

  $effect(() => {
    if (!book) return;
    if (book.format === "epub") {
      if (viewerEl) {
        renderEpub();
      }
    } else if (book.format === "pdf") {
      prefetchFileBlob().then((url) => {
        if (url) {
          loading = false;
        } else {
          error = "Failed to load PDF";
          loading = false;
        }
      });
    } else {
      loading = false;
    }
    return () => {
      if (rendition) {
        rendition.destroy();
        rendition = null;
      }
      if (bookObj) {
        bookObj.destroy();
        bookObj = null;
      }
      if (fileBlobUrl) {
        URL.revokeObjectURL(fileBlobUrl);
        fileBlobUrl = null;
      }
    };
  });

  function setTheme(t: Theme) {
    theme = t;
    if (rendition) rendition.themes.select(t);
    setStoredSettings({ theme, fontSize });
  }

  function changeFontSize(delta: number) {
    fontSize = Math.max(70, Math.min(180, fontSize + delta));
    if (rendition) rendition.themes.fontSize(`${fontSize}%`);
    setStoredSettings({ theme, fontSize });
  }

  function prevPage() {
    if (rendition) rendition.prev();
  }

  function nextPage() {
    if (rendition) rendition.next();
  }

  function goToChapter(href: string) {
    if (rendition) rendition.display(href);
    showToc = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (book?.format !== "epub") return;
    if (showToc) {
      if (e.key === "Escape") showToc = false;
      return;
    }
    if (e.key === "ArrowLeft") prevPage();
    if (e.key === "ArrowRight") nextPage();
    if (e.key === "Escape") goto("/");
  }

  function goBack() {
    goto("/");
  }

  async function downloadFile() {
    const url = await prefetchFileBlob();
    if (!url) return;
    const a = document.createElement("a");
    a.href = url;
    a.download = book?.title ?? "book";
    a.click();
  }
</script>

<svelte:head>
  <title>{book?.title ?? "Reader"} - OpenShelf</title>
</svelte:head>

<svelte:window onkeydown={handleKeydown} />

<div class="reader-app" class:dark={theme === "dark"} class:sepia={theme === "sepia"}>
  <header class="reader-header" class:hidden={!showControls}>
    <button class="icon-btn" onclick={goBack} title="Back to library">&larr;</button>
    <button class="icon-btn" onclick={() => (showToc = !showToc)} title="Contents">&#9776;</button>
    <span class="reader-title">{book?.title ?? "Loading..."}</span>
    <span class="reader-spacer"></span>
    <div class="theme-controls">
      {#each themes as t}
        <button
          class="theme-btn"
          class:active={theme === t.name}
          onclick={() => setTheme(t.name)}
          title={t.name}
        >
          <span class="theme-swatch" style="background: {t.bg}; color: {t.fg};">A</span>
        </button>
      {/each}
    </div>
    <div class="size-controls">
      <button class="size-btn" onclick={() => changeFontSize(-10)}>A-</button>
      <button class="size-btn" onclick={() => changeFontSize(10)}>A+</button>
    </div>
  </header>

  <main class="reader-main" onclick={() => (showControls = !showControls)} role="presentation">
    {#if loading}
      <p class="reader-status">Loading...</p>
    {:else if error}
      <p class="reader-status error">{error}</p>
    {:else if book?.format === "pdf"}
      {#if fileBlobUrl}
        <iframe src={fileBlobUrl} title="PDF viewer" class="pdf-viewer"></iframe>
      {:else}
        <p class="reader-status">Loading PDF...</p>
      {/if}
    {:else if book?.format === "mobi"}
      <div class="unsupported-format">
        <p>MOBI files cannot be displayed in the browser.</p>
        <button class="download-btn" onclick={downloadFile}>Download file</button>
      </div>
    {:else}
      <div class="epub-reader" bind:this={viewerEl}></div>
    {/if}
  </main>

  {#if book?.format === "epub" && !loading && !error}
    <button class="nav-btn nav-prev" onclick={(e) => { e.stopPropagation(); prevPage(); }}>&lsaquo;</button>
    <button class="nav-btn nav-next" onclick={(e) => { e.stopPropagation(); nextPage(); }}>&rsaquo;</button>
    <footer class="reader-footer" class:hidden={!showControls}>
      <span class="progress-text">{progress}%</span>
      <div class="progress-bar">
        <div class="progress-fill" style="width: {progress}%"></div>
      </div>
    </footer>
  {/if}

  {#if showToc && toc.length > 0}
    <div class="toc-overlay" onclick={() => (showToc = false)}>
      <aside class="toc-panel" onclick={(e) => e.stopPropagation()}>
        <h3>Table of Contents</h3>
        <ul class="toc-list">
          {#each toc as item}
            <li>
              <button class="toc-link" onclick={() => goToChapter(item.href)}>
                {item.label}
              </button>
              {#if item.subitems && item.subitems.length > 0}
                <ul class="toc-sublist">
                  {#each item.subitems as sub}
                    <li>
                      <button class="toc-link" onclick={() => goToChapter(sub.href)}>
                        {sub.label}
                      </button>
                    </li>
                  {/each}
                </ul>
              {/if}
            </li>
          {/each}
        </ul>
      </aside>
    </div>
  {/if}
</div>

<style>
  .reader-app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    font-family: system-ui, sans-serif;
    background: #fff;
    color: #1a1a1a;
    transition: background 0.2s, color 0.2s;
  }
  .reader-app.dark {
    background: #1a1a1a;
    color: #d4d4d4;
  }
  .reader-app.sepia {
    background: #f4ecd8;
    color: #3a2f1c;
  }

  .reader-header {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    background: inherit;
    border-bottom: 1px solid rgba(0, 0, 0, 0.1);
    flex-shrink: 0;
    transition: opacity 0.2s, transform 0.2s;
  }
  .reader-app.dark .reader-header {
    border-bottom-color: rgba(255, 255, 255, 0.1);
  }
  .reader-header.hidden {
    opacity: 0;
    transform: translateY(-100%);
    pointer-events: none;
  }

  .icon-btn {
    background: none;
    border: 1px solid rgba(0, 0, 0, 0.15);
    border-radius: 6px;
    padding: 0.35rem 0.6rem;
    cursor: pointer;
    font-size: 1rem;
    color: inherit;
  }
  .icon-btn:hover {
    background: rgba(0, 0, 0, 0.05);
  }
  .reader-app.dark .icon-btn:hover {
    background: rgba(255, 255, 255, 0.05);
  }

  .reader-title {
    font-size: 0.9rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 30vw;
  }

  .reader-spacer {
    flex: 1;
  }

  .theme-controls, .size-controls {
    display: flex;
    gap: 0.25rem;
  }

  .theme-btn, .size-btn {
    background: none;
    border: 1px solid rgba(0, 0, 0, 0.15);
    border-radius: 6px;
    cursor: pointer;
    padding: 0.25rem 0.5rem;
    color: inherit;
    font-size: 0.85rem;
  }
  .theme-btn.active {
    border-color: #4f46e5;
  }

  .theme-swatch {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 600;
    border: 1px solid rgba(0, 0, 0, 0.2);
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

  :global(.epub-reader iframe) {
    border: none;
    width: 100%;
    height: 100%;
    background: transparent;
  }

  .pdf-viewer {
    width: 100%;
    height: 100%;
    border: none;
  }

  .nav-btn {
    position: fixed;
    top: 50%;
    transform: translateY(-50%);
    width: 44px;
    height: 60px;
    border: 1px solid rgba(0, 0, 0, 0.1);
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.7);
    font-size: 1.5rem;
    cursor: pointer;
    color: #555;
    backdrop-filter: blur(8px);
    z-index: 5;
    transition: background 0.15s;
  }
  .nav-btn:hover {
    background: rgba(255, 255, 255, 0.95);
  }
  .reader-app.dark .nav-btn {
    background: rgba(40, 40, 40, 0.7);
    color: #d4d4d4;
    border-color: rgba(255, 255, 255, 0.1);
  }
  .reader-app.dark .nav-btn:hover {
    background: rgba(40, 40, 40, 0.95);
  }
  .nav-prev {
    left: 0.5rem;
  }
  .nav-next {
    right: 0.5rem;
  }

  .reader-footer {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.5rem 1rem;
    background: rgba(255, 255, 255, 0.9);
    backdrop-filter: blur(8px);
    border-top: 1px solid rgba(0, 0, 0, 0.1);
    transition: opacity 0.2s, transform 0.2s;
  }
  .reader-app.dark .reader-footer {
    background: rgba(30, 30, 30, 0.9);
    border-top-color: rgba(255, 255, 255, 0.1);
  }
  .reader-footer.hidden {
    opacity: 0;
    transform: translateY(100%);
    pointer-events: none;
  }
  .progress-text {
    font-size: 0.8rem;
    color: inherit;
    min-width: 3rem;
  }
  .progress-bar {
    flex: 1;
    height: 4px;
    background: rgba(0, 0, 0, 0.1);
    border-radius: 2px;
    overflow: hidden;
  }
  .reader-app.dark .progress-bar {
    background: rgba(255, 255, 255, 0.1);
  }
  .progress-fill {
    height: 100%;
    background: #4f46e5;
    transition: width 0.3s;
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
    border: 1px solid rgba(0, 0, 0, 0.15);
    border-radius: 6px;
    cursor: pointer;
    color: inherit;
    font-size: 0.9rem;
    background: transparent;
  }
  .download-btn:hover {
    background: rgba(0, 0, 0, 0.05);
  }

  .toc-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    display: flex;
    z-index: 50;
  }
  .toc-panel {
    width: 320px;
    max-width: 90vw;
    background: #fff;
    color: #1a1a1a;
    overflow-y: auto;
    padding: 1.5rem;
    box-shadow: 4px 0 20px rgba(0, 0, 0, 0.2);
  }
  .reader-app.dark .toc-panel {
    background: #1a1a1a;
    color: #d4d4d4;
  }
  .reader-app.sepia .toc-panel {
    background: #f4ecd8;
    color: #3a2f1c;
  }
  .toc-panel h3 {
    margin: 0 0 1rem 0;
    font-size: 1.1rem;
  }
  .toc-list, .toc-sublist {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .toc-sublist {
    margin-left: 1rem;
  }
  .toc-link {
    display: block;
    width: 100%;
    text-align: left;
    background: none;
    border: none;
    padding: 0.4rem 0.5rem;
    border-radius: 4px;
    cursor: pointer;
    color: inherit;
    font-size: 0.9rem;
  }
  .toc-link:hover {
    background: rgba(0, 0, 0, 0.05);
  }
  .reader-app.dark .toc-link:hover {
    background: rgba(255, 255, 255, 0.05);
  }
</style>
