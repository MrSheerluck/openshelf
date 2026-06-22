import type { ThemeName, TocItem, Typography } from "./types";
import { themes as defaultThemes, fontFamilyCss } from "./themes";

const WORDS_PER_MINUTE = 250;
const STYLESHEET_KEY = "openshelf-reader";
const PAGE_TURN_TIMEOUT = 2000;

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
  const t = defaultThemes.find((th) => th.name === themeName) ?? defaultThemes[0];
  const fam = fontFamilyCss(typography.fontFamily);
  const align = typography.align === "justify" ? "justify" : "left";
  const borderColor = t.name === "dark" ? "rgba(255,255,255,0.15)" : "rgba(0,0,0,0.12)";

  return `
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

  constructor(options: EpubControllerOptions) {
    this.options = options;
  }

  onProgress(cb: (cfi: string) => void): void {
    this.onCfiChange = cb;
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
