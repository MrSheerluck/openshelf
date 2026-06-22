import type { ThemeName, TocItem, Typography } from "./types";
import { themes as defaultThemes, fontFamilyCss } from "./themes";

const WORDS_PER_MINUTE = 250;

export interface EpubControllerOptions {
  fileUrl: string;
  typography: Typography;
  themeName: ThemeName;
  initialCfi?: string;
}

function estimateReadingMinutes(sections: any[]): number {
  let totalWords = 0;
  for (const section of sections) {
    const text = section.textContent ?? section.innerText ?? "";
    totalWords += text.split(/\s+/).filter(Boolean).length;
  }
  return Math.max(1, Math.ceil(totalWords / WORDS_PER_MINUTE));
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
  private spineItems: any[] = [];

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

      defaultThemes.forEach((t) => {
        this.rendition.themes.register(t.name, {
          "body": {
            "background": `${t.bg} !important`,
            "color": `${t.fg} !important`,
          },
          "p, div, span, li": {
            "color": `${t.fg} !important`,
          },
        });
      });

      this.applyTypography(this.options.typography);
      this.rendition.themes.select(this.options.themeName);

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
        this.estimatedBookMinutes = estimateReadingMinutes(this.bookObj.spine.items ?? []);
      });

      this.rendition.on("relocated", (location: any) => {
        this.progress = Math.round((location.start.percentage ?? 0) * 100);
        this.currentChapter = location.start.href ?? "";
        this.currentSectionIndex = location.start.index ?? 0;
        const cfi = location.start.cfi;
        if (cfi && this.onCfiChange) this.onCfiChange(cfi);
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
    this.rendition?.themes.select(name);
  }

  setTypography(typography: Typography): void {
    this.options.typography = typography;
    this.applyTypography(typography);
  }

  async next(): Promise<void> {
    if (!this.rendition || this.pageTurning) return;
    this.pageTurning = "forward";
    await this.rendition.next();
  }

  async prev(): Promise<void> {
    if (!this.rendition || this.pageTurning) return;
    this.pageTurning = "backward";
    await this.rendition.prev();
  }

  display(href: string): void {
    this.rendition?.display(href);
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

  private applyTypography(typography: Typography): void {
    if (!this.rendition) return;
    this.rendition.themes.fontSize(`${typography.fontSize}%`);
    this.rendition.themes.override("line-height", { "body, p, div, li": { "line-height": `${typography.lineHeight} !important` } }, true);
    this.rendition.themes.override("margin", { "body": { "padding-left": `${typography.margin}px !important`, "padding-right": `${typography.margin}px !important` } }, true);
    this.rendition.themes.override("font-family", { "body, p, div, span, li": { "font-family": `${fontFamilyCss(typography.fontFamily)} !important` } }, true);
    this.rendition.themes.override("text-align", { "body, p, div, li": { "text-align": `${typography.align === "justify" ? "justify" : "left"} !important` } }, true);
  }
}
