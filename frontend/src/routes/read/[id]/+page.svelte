<script lang="ts">
  import { page } from "$app/state";
  import { goto } from "$app/navigation";
  import { api } from "$lib/api";
  import type { Book, ThemeName, Typography } from "$lib/reader/types";
  import { ReaderSettingsStore } from "$lib/reader/settings.svelte";
  import { EpubController } from "$lib/reader/epub.svelte";
  import ReaderHeader from "$lib/components/reader/ReaderHeader.svelte";
  import ReaderFooter from "$lib/components/reader/ReaderFooter.svelte";
  import TocPanel from "$lib/components/reader/TocPanel.svelte";
  import BookViewport from "$lib/components/reader/BookViewport.svelte";
  import TypographyPanel from "$lib/components/reader/TypographyPanel.svelte";

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

  let settings = $state<ReaderSettingsStore | null>(null);
  let controller = $state<EpubController | null>(null);

  let minutesLeft = $derived(
    controller?.estimatedBookMinutes
      ? Math.max(1, Math.round(controller.estimatedBookMinutes * (1 - (controller?.progress ?? 0) / 100)))
      : 0
  );

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

  $effect(() => {
    loadBook();
  });

  $effect(() => {
    if (!book) return;
    if (book.format === "pdf") {
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
      if (fileBlobUrl) {
        URL.revokeObjectURL(fileBlobUrl);
        fileBlobUrl = null;
      }
    };
  });

  function handleViewerMount(el: HTMLDivElement) {
    if (!book || book.format !== "epub" || controller) return;
    if (!settings) {
      settings = new ReaderSettingsStore(id);
    }
    const stored = settings.load();
    const c = new EpubController({
      fileUrl: fileUrl(),
      typography: settings.typography,
      themeName: settings.theme,
      initialCfi: stored.cfi,
    });
    c.onProgress((cfi) => {
      if (settings) settings.save({ cfi });
      saveProgress(cfi);
    });
    controller = c;
    c.mount(el);
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
    if (e.key === "ArrowLeft") prevPage();
    if (e.key === "ArrowRight") nextPage();
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

  function toggleControls() {
    if (showToc || showTypography) return;
    showControls = !showControls;
    if (showControls) scheduleAutoHide();
  }

  let idleTimer: ReturnType<typeof setTimeout> | null = null;

  function scheduleAutoHide() {
    if (idleTimer) clearTimeout(idleTimer);
    idleTimer = setTimeout(() => {
      if (!showToc && !showTypography) showControls = false;
    }, IDLE_HIDE_MS);
  }

  function handleActivity() {
    if (!showControls) showControls = true;
    scheduleAutoHide();
  }

  $effect(() => {
    if (showToc || showTypography) {
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
/>

<div
  class="reader-app"
  class:dark={book?.format === "epub" && settings?.theme === "dark"}
  class:sepia={book?.format === "epub" && settings?.theme === "sepia"}
  class:pdf={book?.format === "pdf"}
>
  <div class="reader-header-wrap" class:hidden={book?.format === "epub" && !showControls}>
    <ReaderHeader
      title={book?.title ?? "Loading..."}
      showTocButton={book?.format === "epub"}
      onBack={goBack}
      onToggleToc={() => (showToc = !showToc)}
      onToggleTypography={() => (showTypography = !showTypography)}
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
    onPrev={prevPage}
    onNext={nextPage}
    onToggleControls={toggleControls}
  />

  {#if book?.format === "epub" && !loading && !error}
    <div class="reader-footer-wrap" class:hidden={!showControls}>
      <ReaderFooter
        progress={controller?.progress ?? 0}
        minutesLeft={minutesLeft}
        totalSections={controller?.totalSections ?? 0}
        currentSectionIndex={controller?.currentSectionIndex ?? 0}
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
