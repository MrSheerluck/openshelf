import type {
  FlattenedTocItem,
  Highlight,
  HighlightColor,
  SearchResult,
  ThemeName,
  TocItem,
  Typography,
} from "./types";
import { fontFamilyCss, themeByName } from "./themes";

const STYLESHEET_KEY = "openshelf-reader";
const PAGE_TURN_TIMEOUT = 2000;
const NAVIGATION_TIMEOUT = 4000;

const HIGHLIGHT_FILLS: Record<HighlightColor, { fill: string; "fill-opacity": string }> = {
  yellow: { fill: "rgb(255, 213, 79)",   "fill-opacity": "0.35" },
  green:  { fill: "rgb(102, 187, 106)",  "fill-opacity": "0.35" },
  blue:   { fill: "rgb(66, 165, 245)",   "fill-opacity": "0.35" },
  pink:   { fill: "rgb(240, 98, 146)",   "fill-opacity": "0.35" },
};

function chapterBasename(href: string): string {
  return (href.split("#")[0] ?? href).replace(/^\.\//, "");
}

function flattenToc(items: TocItem[]): FlattenedTocItem[] {
  const flattened: FlattenedTocItem[] = [];
  let index = 0;
  for (const item of items) {
    if (item.subitems.length === 0) {
      flattened.push({ label: item.label, href: item.href, index });
      index += 1;
      continue;
    }
    flattened.push({ label: item.label, href: item.href, index });
    index += 1;
    for (const subitem of item.subitems) {
      flattened.push({ label: subitem.label, href: subitem.href, index });
      index += 1;
    }
  }
  return flattened;
}

function buildExcerpt(text: string, query: string): string {
  const normalizedText = text.replace(/\s+/g, " ").trim();
  const lowerText = normalizedText.toLowerCase();
  const lowerQuery = query.toLowerCase();
  const matchIndex = lowerText.indexOf(lowerQuery);
  if (matchIndex < 0) {
    return normalizedText.slice(0, 180);
  }
  const start = Math.max(0, matchIndex - 70);
  const end = Math.min(normalizedText.length, matchIndex + query.length + 90);
  const prefix = start > 0 ? "..." : "";
  const suffix = end < normalizedText.length ? "..." : "";
  return `${prefix}${normalizedText.slice(start, end)}${suffix}`;
}

export interface EpubControllerOptions {
  fileUrl: string;
  bookId: string;
  typography: Typography;
  themeName: ThemeName;
  initialCfi?: string;
}

export function buildReaderCss(
  themeName: ThemeName,
  typography: Typography,
  isFixedLayout = false,
): string {
  const t = themeByName(themeName);
  const fam = fontFamilyCss(typography.fontFamily);
  const align = typography.align === "justify" ? "justify" : "left";
  const borderColor = t.isDark ? "rgba(255,255,255,0.12)" : "rgba(0,0,0,0.10)";

  // Fixed-layout EPUBs are effectively authored pages. Reflowing or resetting their
  // positioned content makes illustrations disappear and text drift out of place.
  if (isFixedLayout) {
    return `
      html {
        background: ${t.bg};
        color-scheme: ${t.isDark ? "dark" : "light"};
      }
    `;
  }

  return `
    @font-face {
      font-family: "Literata";
      font-style: normal;
      font-weight: 400;
      src: url("/fonts/literata-latin-400-normal.woff2") format("woff2");
      font-display: swap;
    }
    @font-face {
      font-family: "Literata";
      font-style: italic;
      font-weight: 400;
      src: url("/fonts/literata-latin-400-italic.woff2") format("woff2");
      font-display: swap;
    }
    @font-face {
      font-family: "Literata";
      font-style: normal;
      font-weight: 500;
      src: url("/fonts/literata-latin-500-normal.woff2") format("woff2");
      font-display: swap;
    }
    @font-face {
      font-family: "Literata";
      font-style: normal;
      font-weight: 600;
      src: url("/fonts/literata-latin-600-normal.woff2") format("woff2");
      font-display: swap;
    }
    @font-face {
      font-family: "Literata";
      font-style: normal;
      font-weight: 700;
      src: url("/fonts/literata-latin-700-normal.woff2") format("woff2");
      font-display: swap;
    }
    @font-face {
      font-family: "Literata";
      font-style: italic;
      font-weight: 700;
      src: url("/fonts/literata-latin-700-italic.woff2") format("woff2");
      font-display: swap;
    }
    @font-face {
      font-family: "Andika";
      font-style: normal;
      font-weight: 400;
      src: url("/fonts/andika-latin-400-normal.woff2") format("woff2");
      font-display: swap;
    }
    @font-face {
      font-family: "Andika";
      font-style: italic;
      font-weight: 400;
      src: url("/fonts/andika-latin-400-italic.woff2") format("woff2");
      font-display: swap;
    }
    @font-face {
      font-family: "Andika";
      font-style: normal;
      font-weight: 700;
      src: url("/fonts/andika-latin-700-normal.woff2") format("woff2");
      font-display: swap;
    }
    @font-face {
      font-family: "Andika";
      font-style: italic;
      font-weight: 700;
      src: url("/fonts/andika-latin-700-italic.woff2") format("woff2");
      font-display: swap;
    }
    @font-face {
      font-family: "Libertinus Mono";
      font-style: normal;
      font-weight: 400;
      src: url("/fonts/libertinus-mono-latin-400-normal.woff2") format("woff2");
      font-display: swap;
    }
    @font-face {
      font-family: "Shantell Sans";
      font-style: normal;
      font-weight: 400;
      src: url("/fonts/shantell-sans-latin-400-normal.woff2") format("woff2");
      font-display: swap;
    }
    @font-face {
      font-family: "Shantell Sans";
      font-style: italic;
      font-weight: 400;
      src: url("/fonts/shantell-sans-latin-400-italic.woff2") format("woff2");
      font-display: swap;
    }
    @font-face {
      font-family: "Shantell Sans";
      font-style: normal;
      font-weight: 700;
      src: url("/fonts/shantell-sans-latin-700-normal.woff2") format("woff2");
      font-display: swap;
    }
    @font-face {
      font-family: "Shantell Sans";
      font-style: italic;
      font-weight: 700;
      src: url("/fonts/shantell-sans-latin-700-italic.woff2") format("woff2");
      font-display: swap;
    }
    @font-face {
      font-family: "Noto Sans";
      font-style: normal;
      font-weight: 400;
      src: url("/fonts/noto-sans-latin-400-normal.woff2") format("woff2");
      font-display: swap;
    }
    @font-face {
      font-family: "Noto Sans";
      font-style: italic;
      font-weight: 400;
      src: url("/fonts/noto-sans-latin-400-italic.woff2") format("woff2");
      font-display: swap;
    }
    @font-face {
      font-family: "Noto Sans";
      font-style: normal;
      font-weight: 700;
      src: url("/fonts/noto-sans-latin-700-normal.woff2") format("woff2");
      font-display: swap;
    }
    @font-face {
      font-family: "Noto Sans";
      font-style: italic;
      font-weight: 700;
      src: url("/fonts/noto-sans-latin-700-italic.woff2") format("woff2");
      font-display: swap;
    }

    html {
      font-size: 16px;
      background: ${t.bg};
      margin: 0;
      height: 100%;
      max-width: 100%;
      color-scheme: ${t.isDark ? "dark" : "light"};
    }

    body {
      background: ${t.bg};
      color: ${t.fg};
      font-family: ${fam};
      font-size: ${(typography.fontSize / 100 * 16).toFixed(1)}px;
      -webkit-font-smoothing: antialiased;
      -moz-osx-font-smoothing: grayscale;
      text-rendering: optimizeLegibility;
      font-kerning: normal;
      font-variant-ligatures: common-ligatures;
      box-sizing: border-box !important;
      margin: 0 !important;
      padding-left: ${typography.margin}px !important;
      padding-right: ${typography.margin}px !important;
      height: 100%;
      width: 100%;
      max-width: 100% !important;
      overflow-wrap: break-word;
    }

    *, *::before, *::after {
      box-sizing: border-box !important;
    }

    section, article, main, header, footer, aside, nav, div {
      max-width: 100%;
      min-width: 0;
    }

    p, li, blockquote, td, th, figcaption, dd, dt {
      font-family: ${fam} !important;
      font-size: inherit;
      line-height: ${typography.lineHeight} !important;
      text-align: ${align} !important;
      margin: 0 0 0.35em 0 !important;
      orphans: 2;
      widows: 2;
      word-spacing: 0.01em;
      hyphens: auto;
      -webkit-hyphens: auto;
      -ms-hyphens: auto;
      max-width: 100%;
      overflow-wrap: anywhere;
    }

    h1, h2, h3, h4, h5, h6 {
      font-family: ${fam} !important;
      font-weight: 600 !important;
      line-height: ${typography.lineHeight} !important;
      margin: 0.5em 0 0.15em 0 !important;
      text-indent: 0 !important;
      max-width: 100%;
      overflow-wrap: anywhere;
      break-after: avoid;
      page-break-after: avoid;
    }

    /*
     * Many converted EPUBs create hanging headings with a negative text-indent
     * balanced by a left margin. Reader margins can override only one half of
     * that pair, which sends the title outside the page. Normalize heading-like
     * classes as a unit while leaving ordinary hanging indents alone.
     */
    [class*="head"],
    [class*="title"],
    [epub\\|type~="title"] {
      text-indent: 0 !important;
      margin-left: 0 !important;
      margin-right: 0 !important;
      max-width: 100%;
      overflow-wrap: anywhere;
    }

    div, span, a {
      font-family: ${fam} !important;
    }

    img,
    [class*="illustration"] img,
    [class*="picture"] img,
    [class*="photo"] img {
      max-width: 100% !important;
      max-height: calc(100vh - 2.5rem) !important;
      height: auto !important;
      display: block;
      margin: 0.2em auto !important;
      object-fit: contain;
    }

    svg, picture, canvas {
      max-width: 100% !important;
      max-height: calc(100vh - 2.5rem) !important;
    }

    figure,
    [class~="figure"],
    [role="figure"] {
      max-width: 100%;
      margin: 0.6em auto !important;
      break-inside: auto !important;
      page-break-inside: auto !important;
    }

    figure > img,
    figure > svg,
    figure > picture,
    figure > canvas {
      margin-bottom: 0.35em !important;
    }

    figcaption,
    figure [class*="caption"] {
      display: block;
      clear: both;
      max-width: 100%;
      margin: 0.2em auto 0 !important;
      text-align: center !important;
      font-size: 0.9em !important;
      line-height: 1.4 !important;
      white-space: normal !important;
    }

    pre, code {
      font-family: ui-monospace, "SF Mono", Menlo, Consolas, monospace;
      font-size: 0.9em;
    }

    pre {
      white-space: pre-wrap;
      word-break: break-word;
      overflow-wrap: break-word;
    }

    table {
      width: auto;
      max-width: 100%;
      border-collapse: collapse;
      margin: 0.3em 0 !important;
    }

    blockquote {
      margin: 0.3em 1.5em !important;
      padding: 0 0.8em;
      border-left: 2px solid ${borderColor};
    }

    hr {
      border: none;
      border-top: 1px solid ${borderColor};
      margin: 0.4em 0 !important;
    }

    a {
      color: #4f46e5;
    }

    sup, sub {
      font-size: 0.75em;
      line-height: 0;
    }

    .hl-yellow { background: rgba(255, 213, 79, 0.35) !important; }
    .hl-green  { background: rgba(102, 187, 106, 0.35) !important; }
    .hl-blue   { background: rgba(66, 165, 245, 0.35) !important; }
    .hl-pink   { background: rgba(240, 98, 146, 0.35) !important; }

    ::selection {
      background: rgba(79, 70, 229, 0.25);
    }
  `;
}

export class EpubController {
  bookObj: any = null;
  rendition: any = null;
  progress = $state(0);
  currentChapter = $state("");
  currentCfi = $state("");
  toc = $state<TocItem[]>([]);
  flatToc = $state<FlattenedTocItem[]>([]);
  restoreFailed = $state(false);
  loading = $state(true);
  error = $state("");
  pageTurning = $state<"forward" | "backward" | null>(null);
  totalSections = $state(0);
  currentSectionIndex = $state(0);

  private options: EpubControllerOptions;
  private mounted = false;
  private onCfiChange: ((cfi: string) => void) | null = null;
  private onSelectCb: ((cfiRange: string, text: string, rect: DOMRect) => void) | null = null;
  private onHighlightClickCb: ((cfiRange: string) => void) | null = null;
  private onContentClickCb: (() => void) | null = null;
  private onKeydownCb: ((e: KeyboardEvent) => void) | null = null;
  private navigationPromise: Promise<boolean> = Promise.resolve(true);

  constructor(options: EpubControllerOptions) {
    this.options = options;
  }

  onProgress(cb: (cfi: string) => void): void {
    this.onCfiChange = cb;
  }

  onSelect(cb: (cfiRange: string, text: string, rect: DOMRect) => void): void {
    this.onSelectCb = cb;
  }

  onHighlightClick(cb: (cfiRange: string) => void): void {
    this.onHighlightClickCb = cb;
  }

  onContentClick(cb: () => void): void {
    this.onContentClickCb = cb;
  }

  onKeydown(cb: (e: KeyboardEvent) => void): void {
    this.onKeydownCb = cb;
  }

  async mount(el: HTMLElement): Promise<void> {
    if (this.mounted) return;
    this.mounted = true;

    try {
      const ePub = (await import("epubjs")).default;

      this.bookObj = ePub(this.options.fileUrl, {
        openAs: "epub",
        requestMethod: async (url: string) => {
          const res = await fetch(url, { credentials: "include" });
          if (!res.ok) {
            throw new Error(`Failed to fetch book: ${res.status} ${res.statusText}`);
          }
          return await res.arrayBuffer();
        },
      });

      this.rendition = this.bookObj.renderTo(el, {
        width: "100%",
        height: "100%",
        flow: "paginated",
        spread: "none",
        manager: "default",
      });

      this.rendition.hooks.render.register((view: any) => {
        const contents = view?.contents;
        if (!contents) return;
        if (contents.addStylesheetCss) {
          const css = buildReaderCss(
            this.options.themeName,
            this.options.typography,
            this.isFixedLayout(),
          );
          contents.addStylesheetCss(css, STYLESHEET_KEY);
        }
      });

      // The base tag MUST be in the Document before it's serialized and
      // injected into the iframe. The `content` hook fires on the parsed
      // Document, after epubjs's own `replaceBase` (which we override by
      // writing to a tagged base element). The `render` hook would be too
      // late: the browser has already resolved relative URLs by then.
      this.bookObj.spine.hooks.content.register((doc: any, section: any) => {
        this.injectBaseTag(doc, section);
      });

      if (this.options.initialCfi) {
        this.restoreFailed = false;
        const restored = await this.displayTarget(this.options.initialCfi);
        if (!restored) {
          this.restoreFailed = true;
          await this.displayTarget(undefined);
        }
      } else {
        await this.displayTarget(undefined);
      }

      this.bookObj.ready.then(async () => {
        this.toc = (this.bookObj.navigation.toc ?? []).map((item: any) => ({
          label: item.label,
          href: item.href,
          subitems: (item.subitems ?? []).map((sub: any) => ({
            label: sub.label,
            href: sub.href,
            subitems: [],
          })),
        }));
        this.flatToc = flattenToc(this.toc);
        if (this.bookObj.locations.length() === 0) {
          await this.bookObj.locations.generate(1024);
        }
        this.totalSections = this.bookObj.spine.length;
      });

      this.rendition.on("relocated", (location: any) => {
        this.progress = Math.round((location.start.percentage ?? 0) * 100);
        this.currentChapter = location.start.href ?? "";
        this.currentSectionIndex = location.start.index ?? 0;
        const cfi = location.start.cfi;
        if (cfi) {
          this.currentCfi = cfi;
          if (this.onCfiChange) this.onCfiChange(cfi);
        }
        this.pageTurning = null;
      });

      this.rendition.on("rendered", () => {
        this.pageTurning = null;
      });

      this.rendition.on("selected", (cfiRange: string, contents: any) => {
        try {
          const sel = contents.window.getSelection();
          const text = sel ? sel.toString().trim() : "";
          if (!text || !this.onSelectCb) return;
          const range = sel.getRangeAt(0);
          const rect = range.getBoundingClientRect();
          this.onSelectCb(cfiRange, text, rect);
        } catch {}
      });

      const SWIPE_THRESHOLD = 50;
      let downX = 0, downY = 0, isDown = false;

      this.rendition.on("mousedown", (e: any) => {
        downX = e.clientX ?? 0;
        downY = e.clientY ?? 0;
        isDown = true;

        try {
          const contentsList = this.rendition?.getContents();
          if (contentsList) {
            for (const content of contentsList) {
              const sel = content.window?.getSelection();
              if (sel) sel.removeAllRanges();
            }
          }
        } catch {}
        if (this.onContentClickCb) this.onContentClickCb();
      });

      this.rendition.on("mouseup", (e: any) => {
        if (!isDown) return;
        isDown = false;
        if (this.hasActiveSelection()) return;
        const dx = (e.clientX ?? 0) - downX;
        const dy = (e.clientY ?? 0) - downY;
        if (Math.abs(dx) > SWIPE_THRESHOLD && Math.abs(dx) > Math.abs(dy) * 2) {
          if (dx < 0) this.next();
          else this.prev();
        }
      });

      this.rendition.on("touchstart", (e: any) => {
        const t = e.touches?.[0];
        downX = t?.clientX ?? 0;
        downY = t?.clientY ?? 0;
        isDown = true;

        try {
          const contentsList = this.rendition?.getContents();
          if (contentsList) {
            for (const content of contentsList) {
              const sel = content.window?.getSelection();
              if (sel) sel.removeAllRanges();
            }
          }
        } catch {}
        if (this.onContentClickCb) this.onContentClickCb();
      }, { passive: true });

      this.rendition.on("touchend", (e: any) => {
        if (!isDown) return;
        isDown = false;
        if (this.hasActiveSelection()) return;
        const t = e.changedTouches?.[0];
        if (!t) return;
        const dx = t.clientX - downX;
        const dy = t.clientY - downY;
        if (Math.abs(dx) > SWIPE_THRESHOLD && Math.abs(dx) > Math.abs(dy) * 2) {
          if (dx < 0) this.next();
          else this.prev();
        }
      }, { passive: true });

      this.rendition.on("keydown", (e: any) => {
        if (this.onKeydownCb) this.onKeydownCb(e as KeyboardEvent);
      });

    } catch (e) {
      console.error("EPUB render error:", e);
      this.error = `Failed to render book: ${e instanceof Error ? e.message : String(e)}`;
    } finally {
      this.loading = false;
    }
  }

  setTheme(name: ThemeName): void {
    this.options.themeName = name;
    this.injectToCurrentViews();
  }

  setTypography(typography: Typography): void {
    this.options.typography = typography;
    this.injectToCurrentViews();
  }

  async next(): Promise<void> {
    await this.enqueueNavigation("forward", async () => {
      if (!this.rendition) return false;
      await this.withNavigationTimeout(this.rendition.next());
      return true;
    });
  }

  async prev(): Promise<void> {
    await this.enqueueNavigation("backward", async () => {
      if (!this.rendition) return false;
      await this.withNavigationTimeout(this.rendition.prev());
      return true;
    });
  }

  async display(target: string): Promise<boolean> {
    return this.displayTarget(target);
  }

  currentChapterLabel(): string | null {
    return this.resolveChapterLabel(this.currentSectionIndex, this.currentChapter);
  }

  resolveChapterLabel(chapterIndex: number, href?: string | null): string | null {
    const normalizedHref = href ? chapterBasename(href) : "";
    if (normalizedHref) {
      const hrefMatch = this.flatToc.find((item) => chapterBasename(item.href) === normalizedHref);
      if (hrefMatch?.label) return hrefMatch.label;
    }
    const indexMatch = this.flatToc.find((item) => item.index === chapterIndex);
    return indexMatch?.label ?? null;
  }

  async search(query: string): Promise<SearchResult[]> {
    const trimmed = query.trim();
    if (!trimmed || !this.bookObj?.spine?.spineItems?.length) return [];

    const results: SearchResult[] = [];
    const seen = new Set<string>();
    const lowerQuery = trimmed.toLowerCase();

    for (const [index, section] of this.bookObj.spine.spineItems.entries()) {
      try {
        await section.load(this.bookObj.load.bind(this.bookObj));
        const doc = section.document as Document | undefined;
        const text = doc?.body?.textContent?.replace(/\s+/g, " ").trim() ?? "";
        if (!text) continue;
        if (!text.toLowerCase().includes(lowerQuery)) continue;

        const href = section.href ?? "";
        if (!href || seen.has(href)) continue;
        seen.add(href);

        results.push({
          id: `${index}-${href}`,
          chapterIndex: index,
          chapterLabel: this.resolveChapterLabel(index, href) ?? `Chapter ${index + 1}`,
          excerpt: buildExcerpt(text, trimmed),
          href,
        });

        if (results.length >= 50) break;
      } catch (error) {
        console.warn("Search skipped section", error);
      } finally {
        try {
          section.unload();
        } catch {}
      }
    }

    return results;
  }

  destroy(): void {
    if (this.rendition) {
      this.rendition.destroy();
      this.rendition = null;
    }
    if (this.bookObj) {
      this.bookObj.destroy();
      this.bookObj = null;
    }
    this.mounted = false;
    this.currentCfi = "";
  }

  addHighlight(cfiRange: string, color: HighlightColor): void {
    if (!this.rendition) return;
    const fill = HIGHLIGHT_FILLS[color] ?? HIGHLIGHT_FILLS.yellow;
    this.rendition.annotations.add(
      "highlight",
      cfiRange,
      {},
      () => {
        if (this.onHighlightClickCb) this.onHighlightClickCb(cfiRange);
      },
      `hl-${color}`,
      fill,
    );
  }

  removeHighlight(cfiRange: string): void {
    this.rendition?.annotations.remove(cfiRange, "highlight");
  }

  renderHighlights(highlights: Highlight[]): void {
    if (!this.rendition) return;
    for (const h of highlights) {
      this.addHighlight(h.cfiRange, h.color);
    }
  }

  clearSelection(): void {
    if (!this.rendition) return;
    try {
      const contents = this.rendition.getContents();
      for (const c of contents) {
        const sel = c.window?.getSelection();
        if (sel) sel.removeAllRanges();
      }
    } catch {}
  }

  private hasActiveSelection(): boolean {
    if (!this.rendition) return false;
    try {
      const contentsList = this.rendition.getContents();
      for (const contents of contentsList) {
        const selection = contents.window?.getSelection();
        if (selection && !selection.isCollapsed && selection.toString().trim()) {
          return true;
        }
      }
    } catch {}
    return false;
  }

  private injectToCurrentViews(): void {
    if (!this.rendition) return;
    const css = buildReaderCss(
      this.options.themeName,
      this.options.typography,
      this.isFixedLayout(),
    );
    const contentsList = this.rendition.getContents();
    for (const contents of contentsList) {
      if (contents && contents.addStylesheetCss) {
        contents.addStylesheetCss(css, STYLESHEET_KEY);
      }
    }
  }

  private injectBaseTag(doc: any, section: any): void {
    if (!doc) return;
    const head = doc.querySelector?.("head");
    if (!head) return;

    const sectionHref: string | undefined = section?.href;
    const baseHref = sectionHref
      ? `/api/books/${this.options.bookId}/resource/${
          (() => {
            const cleanHref = sectionHref.split("#")[0] ?? sectionHref;
            const dir = cleanHref.split("/").slice(0, -1).join("/");
            return dir ? dir + "/" : "";
          })()
        }`
      : `/api/books/${this.options.bookId}/resource/`;

    // Override whatever base href epubjs's own `replaceBase` set. We tag
    // the element so we can find it again on re-renders.
    let base = head.querySelector("base[data-openshelf]");
    if (!base) {
      base = doc.createElement("base");
      base.setAttribute("data-openshelf", "true");
      head.prepend(base);
    }
    base.setAttribute("href", baseHref);
  }

  private isFixedLayout(): boolean {
    return this.rendition?.layout?.()?.name === "pre-paginated";
  }

  private async displayTarget(target?: string): Promise<boolean> {
    return this.enqueueNavigation("forward", async () => {
      if (!this.rendition) return false;
      await this.withNavigationTimeout(
        target ? this.rendition.display(target) : this.rendition.display(),
      );
      return true;
    });
  }

  private async enqueueNavigation(
    direction: "forward" | "backward",
    action: () => Promise<boolean>,
  ): Promise<boolean> {
    const run = async () => {
      this.pageTurning = direction;
      const timeout = setTimeout(() => {
        this.pageTurning = null;
      }, PAGE_TURN_TIMEOUT);
      try {
        return await action();
      } catch (error) {
        console.error("Reader navigation failed", error);
        return false;
      } finally {
        clearTimeout(timeout);
        this.pageTurning = null;
      }
    };

    this.navigationPromise = this.navigationPromise.then(run, run);
    return this.navigationPromise;
  }

  private async withNavigationTimeout<T>(promise: Promise<T>): Promise<T> {
    return await Promise.race([
      promise,
      new Promise<never>((_, reject) => {
        setTimeout(() => {
          reject(new Error("Navigation timed out"));
        }, NAVIGATION_TIMEOUT);
      }),
    ]);
  }
}
