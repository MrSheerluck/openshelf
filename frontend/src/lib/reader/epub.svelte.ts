import type { ThemeName, TocItem, Typography, Highlight, HighlightColor } from "./types";
import { themes as defaultThemes, fontFamilyCss, themeByName } from "./themes";

const WORDS_PER_MINUTE = 250;
const STYLESHEET_KEY = "openshelf-reader";
const PAGE_TURN_TIMEOUT = 2000;

const HIGHLIGHT_FILLS: Record<HighlightColor, { fill: string; "fill-opacity": string }> = {
  yellow: { fill: "rgb(255, 213, 79)",   "fill-opacity": "0.35" },
  green:  { fill: "rgb(102, 187, 106)",  "fill-opacity": "0.35" },
  blue:   { fill: "rgb(66, 165, 245)",   "fill-opacity": "0.35" },
  pink:   { fill: "rgb(240, 98, 146)",   "fill-opacity": "0.35" },
};

export interface EpubControllerOptions {
  fileUrl: string;
  typography: Typography;
  themeName: ThemeName;
  initialCfi?: string;
}

function estimateReadingMinutes(locations: any, bookObj: any): number {
  try {
    const total = bookObj.locations.length();
    if (total > 0) {
      return Math.max(1, Math.ceil(total / 4));
    }
  } catch {}
  return 0;
}

function buildReaderCss(themeName: ThemeName, typography: Typography): string {
  const t = themeByName(themeName);
  const fam = fontFamilyCss(typography.fontFamily);
  const align = typography.align === "justify" ? "justify" : "left";
  const borderColor = t.isDark ? "rgba(255,255,255,0.12)" : "rgba(0,0,0,0.10)";

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
      margin: 0;
      padding: 0 ${typography.margin}px;
      padding-top: 0 !important;
      padding-bottom: 0 !important;
      height: 100%;
      width: 100%;
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
    }

    h1, h2, h3, h4, h5, h6 {
      font-family: ${fam} !important;
      font-weight: 600 !important;
      line-height: ${typography.lineHeight} !important;
      margin: 0.5em 0 0.15em 0 !important;
    }

    div, span, a {
      font-family: ${fam} !important;
    }

    img, svg, figure, picture, canvas,
    [class*="img"], [class*="image"], [class*="figure"],
    [class*="illustration"], [class*="picture"], [class*="photo"] {
      max-width: 100%;
      max-height: 85vh;
      height: auto;
      width: auto;
      display: block;
      margin: 0.2em auto !important;
      break-inside: auto;
      page-break-inside: auto;
      object-fit: scale-down;
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

    body > :first-child,
    body > :first-child > :first-child,
    body > :first-child > :first-child > :first-child,
    body > section:first-child,
    body > article:first-child,
    body > div:first-child,
    body > main:first-child,
    body > section:first-child > :first-child,
    body > div:first-child > :first-child,
    body > article:first-child > :first-child {
      margin-top: 0 !important;
      padding-top: 0 !important;
    }

    h1:first-child, h2:first-child, h3:first-child,
    h1:first-of-type, h2:first-of-type, h3:first-of-type {
      margin-top: 0 !important;
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
  toc = $state<TocItem[]>([]);
  loading = $state(true);
  error = $state("");
  pageTurning = $state<"forward" | "backward" | null>(null);
  totalSections = $state(0);
  currentSectionIndex = $state(0);
  estimatedBookMinutes = $state(0);

  private options: EpubControllerOptions;
  private mounted = false;
  private onCfiChange: ((cfi: string) => void) | null = null;
  private onSelectCb: ((cfiRange: string, text: string, rect: DOMRect) => void) | null = null;
  private onHighlightClickCb: ((cfiRange: string) => void) | null = null;

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

      const contentHooks = this.rendition.hooks.content.hooks as Function[];
      for (let i = contentHooks.length - 1; i >= 0; i--) {
        const fn = contentHooks[i];
        if (fn.name === "bound adjustImages" || fn.toString().includes("break-inside")) {
          contentHooks.splice(i, 1);
          break;
        }
      }

      this.rendition.hooks.render.register((view: any) => {
        const contents = view?.contents;
        if (contents && contents.addStylesheetCss) {
          const css = buildReaderCss(this.options.themeName, this.options.typography);
          contents.addStylesheetCss(css, STYLESHEET_KEY);
        }
      });

      await this.rendition.display();

      if (this.options.initialCfi) {
        await this.rendition.display(this.options.initialCfi);
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
        if (this.bookObj.locations.length() === 0) {
          await this.bookObj.locations.generate(1024);
        }
        this.totalSections = this.bookObj.spine.length;
        this.estimatedBookMinutes = estimateReadingMinutes(this.bookObj.locations, this.bookObj);
      });

      this.rendition.on("relocated", (location: any) => {
        this.progress = Math.round((location.start.percentage ?? 0) * 100);
        this.currentChapter = location.start.href ?? "";
        this.currentSectionIndex = location.start.index ?? 0;
        const cfi = location.start.cfi;
        if (cfi && this.onCfiChange) this.onCfiChange(cfi);
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
      });

      this.rendition.on("mouseup", (e: any) => {
        if (!isDown) return;
        isDown = false;
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
      }, { passive: true });

      this.rendition.on("touchend", (e: any) => {
        if (!isDown) return;
        isDown = false;
        const t = e.changedTouches?.[0];
        if (!t) return;
        const dx = t.clientX - downX;
        const dy = t.clientY - downY;
        if (Math.abs(dx) > SWIPE_THRESHOLD && Math.abs(dx) > Math.abs(dy) * 2) {
          if (dx < 0) this.next();
          else this.prev();
        }
      }, { passive: true });
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
    if (!this.rendition || this.pageTurning) return;
    this.pageTurning = "forward";
    const timeout = setTimeout(() => { this.pageTurning = null; }, PAGE_TURN_TIMEOUT);
    try {
      await this.rendition.next();
    } finally {
      clearTimeout(timeout);
    }
  }

  async prev(): Promise<void> {
    if (!this.rendition || this.pageTurning) return;
    this.pageTurning = "backward";
    const timeout = setTimeout(() => { this.pageTurning = null; }, PAGE_TURN_TIMEOUT);
    try {
      await this.rendition.prev();
    } finally {
      clearTimeout(timeout);
    }
  }

  async display(href: string): Promise<void> {
    if (!this.rendition || this.pageTurning) return;
    this.pageTurning = "forward";
    const timeout = setTimeout(() => { this.pageTurning = null; }, PAGE_TURN_TIMEOUT);
    try {
      await this.rendition.display(href);
    } finally {
      clearTimeout(timeout);
    }
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

  private injectToCurrentViews(): void {
    if (!this.rendition) return;
    const css = buildReaderCss(this.options.themeName, this.options.typography);
    const contentsList = this.rendition.getContents();
    for (const contents of contentsList) {
      if (contents && contents.addStylesheetCss) {
        contents.addStylesheetCss(css, STYLESHEET_KEY);
      }
    }
  }
}
