<script lang="ts">
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import { api } from "$lib/api";
  import type {
    Book,
    ThemeName,
    Typography,
    HighlightColor,
    Highlight,
    SearchResult,
  } from "$lib/reader/types";
  import { ReaderSettingsStore } from "$lib/reader/settings.svelte";
  import { HighlightsStore } from "$lib/reader/highlights.svelte";
  import { BookmarksStore } from "$lib/reader/bookmarks.svelte";
  import { EpubController } from "$lib/reader/epub.svelte";
  import ReaderHeader from "$lib/components/reader/ReaderHeader.svelte";
  import ReaderFooter from "$lib/components/reader/ReaderFooter.svelte";
  import TocPanel from "$lib/components/reader/TocPanel.svelte";
  import BookViewport from "$lib/components/reader/BookViewport.svelte";
  import TypographyPanel from "$lib/components/reader/TypographyPanel.svelte";
  import SelectionMenu from "$lib/components/reader/SelectionMenu.svelte";
  import DictionaryPopup from "$lib/components/reader/DictionaryPopup.svelte";
  import NoteEditor from "$lib/components/reader/NoteEditor.svelte";
  import HighlightsList from "$lib/components/reader/HighlightsList.svelte";
  import BookmarksPanel from "$lib/components/reader/BookmarksPanel.svelte";
  import SearchPanel from "$lib/components/reader/SearchPanel.svelte";

  const API_URL = import.meta.env.VITE_API_URL ?? "http://localhost:3001";
  const IDLE_HIDE_MS = 3000;
  const id = $derived(String(page.params.id));

  let book = $state<Book | null>(null);
  let loading = $state(true);
  let error = $state("");
  let fileBlobUrl = $state<string | null>(null);
  let showControls = $state(true);
  let showToc = $state(false);
  let showTypography = $state(false);
  let showHighlights = $state(false);
  let showBookmarks = $state(false);
  let showSearch = $state(false);

  let settings = $state<ReaderSettingsStore | null>(null);
  let highlightsStore = $state<HighlightsStore | null>(null);
  let bookmarksStore = $state<BookmarksStore | null>(null);
  let controller = $state<EpubController | null>(null);
  let viewerMountStarted = false;

  type Selection = { cfiRange: string; text: string; x: number; y: number } | null;
  let selection = $state<Selection>(null);
  let dictWord = $state<{ word: string; x: number; y: number } | null>(null);
  let noteEditor = $state<{ highlight: Highlight; x: number; y: number } | null>(null);
  let searchQuery = $state("");
  let searchResults = $state<SearchResult[]>([]);
  let searching = $state(false);
  let searchTimer: ReturnType<typeof setTimeout> | null = null;

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
    loading = true;
    error = "";
    try {
      const res = await api(`/api/books/${id}`);
      if (res.ok) {
        book = await res.json();
      } else {
        error = "Book not found";
        loading = false;
      }
    } catch {
      error = "Failed to load book";
      loading = false;
    }
  }

  $effect(() => {
    loadBook();
  });

  $effect(() => {
    if (!book) return;
    if (book.format === "pdf" || book.format === "epub") {
      prefetchFileBlob().then((url) => {
        if (url) {
          loading = false;
        } else {
          error = `Failed to load ${book?.format === "pdf" ? "PDF" : "EPUB"}`;
          loading = false;
        }
      });
    } else {
      loading = false;
    }
    return () => {
      if (fileBlobUrl) {
        URL.revokeObjectURL(fileBlobUrl);
        fileBlobUrl = null;
      }
    };
  });

  async function handleViewerMount(el: HTMLDivElement) {
    if (!book || book.format !== "epub" || !fileBlobUrl || controller || viewerMountStarted) return;
    viewerMountStarted = true;
    if (!settings) {
      settings = new ReaderSettingsStore(id);
    }
    if (!highlightsStore) {
      highlightsStore = new HighlightsStore(id);
    }
    if (!bookmarksStore) {
      bookmarksStore = new BookmarksStore(id);
    }
    const stored = await settings.load();
    await highlightsStore.load();
    await bookmarksStore.load();
    const c = new EpubController({
      fileUrl: fileBlobUrl,
      bookId: id,
      typography: settings.typography,
      themeName: settings.theme,
      initialCfi: stored.cfi,
    });
    c.onProgress((cfi) => {
      if (settings) settings.save({ cfi });
      saveProgress(cfi);
    });
    c.onSelect((cfiRange, text, rect) => {
      selection = { cfiRange, text, x: rect.left + rect.width / 2, y: rect.top };
    });
    c.onHighlightClick((cfiRange) => {
      if (!highlightsStore) return;
      const h = highlightsStore.findByCfi(cfiRange);
      if (h) {
        let x = window.innerWidth / 2, y = window.innerHeight / 2;
        try {
          const ann = c.rendition?.annotations?._annotations?.[encodeURI(cfiRange + "highlight")];
          if (ann?.mark?.element) {
            const r = ann.mark.element.getBoundingClientRect();
            x = r.left + r.width / 2;
            y = r.top;
          }
        } catch {}
        noteEditor = { highlight: h, x, y };
      }
    });
    controller = c;
    await c.mount(el);
    if (c.error) {
      error = c.error;
      return;
    }
    if (c.restoreFailed) {
      settings?.clearCfi();
    }
    if (highlightsStore) {
      c.renderHighlights(highlightsStore.highlights);
      syncHighlightChapterLabels();
    }
  }

  function setTheme(t: ThemeName) {
    if (!settings) return;
    settings.theme = t;
    controller?.setTheme(t);
    settings.save();
  }

  function setTypography(typography: Typography) {
    if (!settings) return;
    settings.typography = typography;
    controller?.setTypography(typography);
    settings.save();
  }

  function handleHighlight(color: HighlightColor) {
    if (!selection || !highlightsStore || !controller) return;
    highlightsStore.add(
      selection.cfiRange,
      selection.text,
      color,
      controller.currentSectionIndex,
      controller.currentChapterLabel(),
    );
    controller.addHighlight(selection.cfiRange, color);
    controller.clearSelection();
    selection = null;
  }

  function handleDictionary() {
    if (!selection) return;
    const word = selection.text.split(/\s+/)[0]?.replace(/[^\w'-]/g, "");
    if (word) {
      dictWord = { word, x: selection.x, y: selection.y };
    }
    selection = null;
  }

  function handleCopy() {
    selection = null;
  }

  function closeSelection() {
    selection = null;
    controller?.clearSelection();
  }

  function handleSaveNote(note: string | null) {
    if (!noteEditor || !highlightsStore) return;
    highlightsStore.updateNote(noteEditor.highlight.id, note);
    noteEditor = null;
  }

  function handleSetHighlightColor(color: HighlightColor) {
    if (!noteEditor || !highlightsStore || !controller) return;
    highlightsStore.updateColor(noteEditor.highlight.id, color);
    noteEditor.highlight.color = color;
    controller.removeHighlight(noteEditor.highlight.cfiRange);
    controller.addHighlight(noteEditor.highlight.cfiRange, color);
  }

  function handleDeleteHighlight() {
    if (!noteEditor || !highlightsStore || !controller) return;
    highlightsStore.remove(noteEditor.highlight.id);
    controller.removeHighlight(noteEditor.highlight.cfiRange);
    noteEditor = null;
  }

  function handleDeleteHighlightFromList(id: string) {
    if (!highlightsStore || !controller) return;
    const h = highlightsStore.highlights.find((x) => x.id === id);
    if (h) controller.removeHighlight(h.cfiRange);
    highlightsStore.remove(id);
  }

  function handleHighlightSelect(cfiRange: string) {
    controller?.display(cfiRange);
    showHighlights = false;
  }

  function handleBookmarkSelect(cfi: string) {
    controller?.display(cfi);
    showBookmarks = false;
  }

  async function toggleBookmark() {
    if (!controller?.currentCfi || !bookmarksStore) return;
    const existing = bookmarksStore.findByCfi(controller.currentCfi);
    if (existing) {
      await bookmarksStore.remove(existing.id);
      return;
    }
    await bookmarksStore.add(
      controller.currentCfi,
      controller.currentSectionIndex,
      controller.currentChapterLabel(),
    );
  }

  function jumpBookmark(direction: 1 | -1) {
    if (!controller?.currentCfi || !bookmarksStore || bookmarksStore.bookmarks.length === 0) return;
    const sorted = [...bookmarksStore.bookmarks].sort((a, b) => a.chapterIndex - b.chapterIndex || a.createdAt - b.createdAt);
    const currentIndex = sorted.findIndex((bookmark) => bookmark.cfi === controller?.currentCfi);
    const anchorIndex = currentIndex >= 0 ? currentIndex : (direction > 0 ? -1 : 0);
    const nextIndex = direction > 0
      ? (anchorIndex + 1) % sorted.length
      : (anchorIndex <= 0 ? sorted.length - 1 : anchorIndex - 1);
    controller.display(sorted[nextIndex].cfi);
  }

  async function runSearch(query: string) {
    if (!controller || query.trim().length < 2) {
      searchResults = [];
      searching = false;
      return;
    }
    searching = true;
    searchResults = await controller.search(query);
    searching = false;
  }

  function syncHighlightChapterLabels() {
    if (!controller || !highlightsStore) return;
    for (const highlight of highlightsStore.highlights) {
      const label = controller.resolveChapterLabel(highlight.chapterIndex, highlight.cfiRange);
      highlightsStore.setChapterLabel(highlight.id, label);
    }
  }

  function prevPage() {
    controller?.prev();
  }

  function nextPage() {
    controller?.next();
  }

  function goToChapter(href: string) {
    controller?.display(href);
    showToc = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (showTypography && e.key === "Escape") {
      showTypography = false;
      return;
    }
    if (book?.format !== "epub") return;
    if (showToc) {
      if (e.key === "Escape") showToc = false;
      return;
    }
    if (showHighlights) {
      if (e.key === "Escape") showHighlights = false;
      return;
    }
    if (showBookmarks) {
      if (e.key === "Escape") showBookmarks = false;
      return;
    }
    if (showSearch) {
      if (e.key === "Escape") showSearch = false;
      return;
    }
    if (selection || dictWord || noteEditor) {
      if (e.key === "Escape") {
        selection = null;
        dictWord = null;
        noteEditor = null;
      }
      return;
    }
    if (e.key === "ArrowLeft") prevPage();
    if (e.key === "ArrowRight") nextPage();
    if (e.key.toLowerCase() === "b") toggleBookmark();
    if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "f") {
      e.preventDefault();
      showSearch = true;
    }
    if (e.key === "Escape") goto("/");
  }

  function goBack() {
    goto("/");
  }

  function downloadFile() {
    prefetchFileBlob().then((url) => {
      if (!url) return;
      const a = document.createElement("a");
      a.href = url;
      a.download = book?.title ?? "book";
      a.click();
    });
  }

  let idleTimer: ReturnType<typeof setTimeout> | null = null;

  function scheduleAutoHide() {
    if (idleTimer) clearTimeout(idleTimer);
    idleTimer = setTimeout(() => {
      if (!showToc && !showTypography && !showHighlights) showControls = false;
    }, IDLE_HIDE_MS);
  }

  function handleActivity() {
    if (!showControls) showControls = true;
    scheduleAutoHide();
  }

  $effect(() => {
    if (showToc || showTypography || showHighlights || showBookmarks || showSearch) {
      if (idleTimer) {
        clearTimeout(idleTimer);
        idleTimer = null;
      }
      showControls = true;
      return;
    }
    scheduleAutoHide();
    return () => {
      if (idleTimer) {
        clearTimeout(idleTimer);
        idleTimer = null;
      }
    };
  });

  $effect(() => {
    if (searchTimer) {
      clearTimeout(searchTimer);
      searchTimer = null;
    }
    if (!showSearch) {
      searching = false;
      return;
    }
    searchTimer = setTimeout(() => {
      runSearch(searchQuery);
    }, 220);
    return () => {
      if (searchTimer) {
        clearTimeout(searchTimer);
        searchTimer = null;
      }
    };
  });

  $effect(() => {
    if (!controller || !highlightsStore) return;
    controller.flatToc.length;
    highlightsStore.highlights.length;
    syncHighlightChapterLabels();
  });

  $effect(() => {
    return () => {
      if (controller) {
        controller.destroy();
        controller = null;
      }
    };
  });
</script>

<svelte:head>
  <title>{book?.title ?? "Reader"} - OpenShelf</title>
</svelte:head>

<svelte:window
  onkeydown={handleKeydown}
  onpointermove={handleActivity}
  ontouchstart={handleActivity}
  onclick={() => { if (selection) closeSelection(); if (dictWord) dictWord = null; if (noteEditor) noteEditor = null; }}
/>

<div
  class="reader-app"
  class:dark={book?.format === "epub" && settings?.theme === "dark"}
  class:sepia={book?.format === "epub" && settings?.theme === "sepia"}
  class:cream={book?.format === "epub" && settings?.theme === "cream"}
  class:green={book?.format === "epub" && settings?.theme === "green"}
  class:night={book?.format === "epub" && settings?.theme === "night"}
  class:pdf={book?.format === "pdf"}
>
  <div class="reader-header-wrap" class:hidden={book?.format === "epub" && !showControls}>
    <ReaderHeader
      title={book?.title ?? "Loading..."}
      showTocButton={book?.format === "epub"}
      isBookmarked={!!(controller?.currentCfi && bookmarksStore?.findByCfi(controller.currentCfi))}
      onBack={goBack}
      onToggleSearch={() => (showSearch = !showSearch)}
      onToggleBookmarks={() => (showBookmarks = !showBookmarks)}
      onToggleBookmark={toggleBookmark}
      onToggleToc={() => (showToc = !showToc)}
      onToggleTypography={() => (showTypography = !showTypography)}
      onToggleHighlights={() => (showHighlights = !showHighlights)}
    />
  </div>

  <BookViewport
    {book}
    {fileBlobUrl}
    {loading}
    {error}
    pageTurning={controller?.pageTurning ?? null}
    onViewerMount={handleViewerMount}
    onDownload={downloadFile}
  />

  {#if book?.format === "epub" && !loading && !error}
    <div class="reader-footer-wrap" class:hidden={!showControls}>
      <ReaderFooter
        progress={controller?.progress ?? 0}
        totalSections={controller?.totalSections ?? 0}
        currentSectionIndex={controller?.currentSectionIndex ?? 0}
        canJumpBookmarks={(bookmarksStore?.bookmarks.length ?? 0) > 0}
        onPrevBookmark={() => jumpBookmark(-1)}
        onNextBookmark={() => jumpBookmark(1)}
      />
    </div>
  {/if}

  {#if showToc && controller && controller.toc.length > 0 && book?.format === "epub"}
    <TocPanel
      items={controller.toc}
      currentChapter={controller.currentChapter}
      onSelect={goToChapter}
      onClose={() => (showToc = false)}
    />
  {/if}

  {#if showTypography && settings && book?.format === "epub"}
    <TypographyPanel
      typography={settings.typography}
      theme={settings.theme}
      progress={controller?.progress ?? 0}
      onChange={setTypography}
      onSetTheme={setTheme}
      onClose={() => (showTypography = false)}
    />
  {/if}

  {#if showHighlights && highlightsStore && book?.format === "epub"}
    <HighlightsList
      highlights={highlightsStore.highlights}
      bookId={id}
      bookTitle={book?.title ?? "book"}
      onSelect={handleHighlightSelect}
      onDelete={handleDeleteHighlightFromList}
      onClose={() => (showHighlights = false)}
    />
  {/if}

  {#if showBookmarks && bookmarksStore && book?.format === "epub"}
    <BookmarksPanel
      bookmarks={bookmarksStore.bookmarks}
      onSelect={handleBookmarkSelect}
      onDelete={(bookmarkId) => bookmarksStore?.remove(bookmarkId)}
      onClose={() => (showBookmarks = false)}
    />
  {/if}

  {#if showSearch && controller && book?.format === "epub"}
    <SearchPanel
      query={searchQuery}
      {searching}
      results={searchResults}
      onQueryChange={(query) => (searchQuery = query)}
      onSelect={(href) => {
        controller?.display(href);
        showSearch = false;
      }}
      onClose={() => (showSearch = false)}
    />
  {/if}

  {#if selection}
    <SelectionMenu
      x={selection.x}
      y={selection.y}
      text={selection.text}
      onHighlight={handleHighlight}
      onDictionary={handleDictionary}
      onCopy={handleCopy}
      onClose={closeSelection}
    />
  {/if}

  {#if dictWord}
    <DictionaryPopup
      word={dictWord.word}
      x={dictWord.x}
      y={dictWord.y}
      onClose={() => (dictWord = null)}
    />
  {/if}

  {#if noteEditor}
    <NoteEditor
      highlight={noteEditor.highlight}
      x={noteEditor.x}
      y={noteEditor.y}
      onSave={handleSaveNote}
      onSetColor={handleSetHighlightColor}
      onDelete={handleDeleteHighlight}
      onClose={() => (noteEditor = null)}
    />
  {/if}
</div>

<style>
  .reader-app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    font-family: system-ui, sans-serif;
    background: var(--reader-bg, #fff);
    color: var(--reader-fg, #1a1a1a);
    transition: background 0.2s, color 0.2s;
    --reader-bg: #ffffff;
    --reader-fg: #1a1a1a;
    --reader-border: rgba(0, 0, 0, 0.1);
    --reader-hover: rgba(0, 0, 0, 0.05);
    --reader-panel-bg: #ffffff;
    --reader-panel-fg: #1a1a1a;
    --reader-muted: #9ca3af;
    --reader-progress-track: rgba(0, 0, 0, 0.1);
  }
  .reader-app.dark {
    --reader-bg: #1a1a1a;
    --reader-fg: #d4d4d4;
    --reader-border: rgba(255, 255, 255, 0.1);
    --reader-hover: rgba(255, 255, 255, 0.05);
    --reader-panel-bg: #1a1a1a;
    --reader-panel-fg: #d4d4d4;
    --reader-muted: #9ca3af;
    --reader-progress-track: rgba(255, 255, 255, 0.1);
  }
  .reader-app.night {
    --reader-bg: #0a0a0a;
    --reader-fg: #c0c0c0;
    --reader-border: rgba(255, 255, 255, 0.08);
    --reader-hover: rgba(255, 255, 255, 0.04);
    --reader-panel-bg: #0a0a0a;
    --reader-panel-fg: #c0c0c0;
    --reader-muted: #777;
    --reader-progress-track: rgba(255, 255, 255, 0.08);
  }
  .reader-app.sepia {
    --reader-bg: #f4ecd8;
    --reader-fg: #3a2f1c;
    --reader-panel-bg: #f4ecd8;
    --reader-panel-fg: #3a2f1c;
    --reader-muted: #8b7d6b;
    --reader-border: rgba(58, 47, 28, 0.12);
    --reader-hover: rgba(58, 47, 28, 0.06);
    --reader-progress-track: rgba(58, 47, 28, 0.12);
  }
  .reader-app.cream {
    --reader-bg: #f7f3e9;
    --reader-fg: #3a3226;
    --reader-panel-bg: #f7f3e9;
    --reader-panel-fg: #3a3226;
    --reader-muted: #8a8270;
    --reader-border: rgba(58, 50, 38, 0.1);
    --reader-hover: rgba(58, 50, 38, 0.05);
    --reader-progress-track: rgba(58, 50, 38, 0.1);
  }
  .reader-app.green {
    --reader-bg: #e4ede4;
    --reader-fg: #2d3a2d;
    --reader-panel-bg: #e4ede4;
    --reader-panel-fg: #2d3a2d;
    --reader-muted: #7a8a7a;
    --reader-border: rgba(45, 58, 45, 0.1);
    --reader-hover: rgba(45, 58, 45, 0.05);
    --reader-progress-track: rgba(45, 58, 45, 0.1);
  }
  .reader-app.pdf {
    --reader-bg: #525659;
    --reader-fg: #d4d4d4;
  }

  .reader-header-wrap {
    position: relative;
    z-index: 10;
    transition: opacity 0.25s, transform 0.25s;
  }
  .reader-header-wrap.hidden {
    opacity: 0;
    transform: translateY(-100%);
    pointer-events: none;
  }

  .reader-footer-wrap {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    z-index: 4;
    transition: opacity 0.25s, transform 0.25s;
  }
  .reader-footer-wrap.hidden {
    opacity: 0;
    transform: translateY(100%);
    pointer-events: none;
  }
</style>
